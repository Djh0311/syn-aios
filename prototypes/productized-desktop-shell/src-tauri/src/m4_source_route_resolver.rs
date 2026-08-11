//! Closed, read-only resolver for server-minted M4 source routes.
//!
//! The renderer supplies only the opaque route capability.  M4 recovers its
//! current admission and provenance; the registered owner store then rebuilds
//! the native publication and finite target.  No path, URL, view name,
//! callback, executable payload, or owner write capability crosses this seam.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::m4_secretary_domain::{
    m4_internal_id, m4_primary_scope_ref, M4_WORKFLOW_ATTENTION_SOURCE_TYPE,
};
use crate::m4_secretary_repository::{
    M4CurrentSourceRouteCandidate, M4SecretaryRepositoryError, M4SecretarySqliteRepository,
};
use crate::m4_source_owner_schema::{
    M4SourceOwnerPublicationExpectationV1, M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
    M4_PROPOSAL_SOURCE_OWNER_REF, M4_WORK_ITEM_SOURCE_ADAPTER_ID, M4_WORK_ITEM_SOURCE_OWNER_REF,
};
use crate::workbench_sqlite_repository::M4SourceOwnerNavigationTargetRead;

pub(crate) const M4_SOURCE_ROUTE_RESOLUTION_SCHEMA: &str =
    "syn.m4.secretary.source-route-resolution.v1";
pub(crate) const M4_SOURCE_ROUTE_INVALID: &str = "M4_SOURCE_ROUTE_INVALID";
pub(crate) const M4_SOURCE_ROUTE_TAMPERED: &str = "M4_SOURCE_ROUTE_TAMPERED";
pub(crate) const M4_SOURCE_OWNER_UNREGISTERED: &str = "M4_SOURCE_OWNER_UNREGISTERED";
pub(crate) const M4_SOURCE_TYPE_UNREGISTERED: &str = "M4_SOURCE_TYPE_UNREGISTERED";
pub(crate) const M4_SOURCE_SCOPE_MISMATCH: &str = "M4_SOURCE_SCOPE_MISMATCH";
pub(crate) const M4_SOURCE_ROUTE_STALE: &str = "M4_SOURCE_ROUTE_STALE";
pub(crate) const M4_SOURCE_REVISION_MISMATCH: &str = "M4_SOURCE_REVISION_MISMATCH";
pub(crate) const M4_SOURCE_TARGET_MISSING: &str = "M4_SOURCE_TARGET_MISSING";
pub(crate) const M4_SOURCE_TARGET_INTEGRITY_FAILED: &str = "M4_SOURCE_TARGET_INTEGRITY_FAILED";
pub(crate) const M4_SOURCE_ROUTE_REGISTRY_UNAVAILABLE: &str =
    "M4_SOURCE_ROUTE_REGISTRY_UNAVAILABLE";
pub(crate) const M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE: &str =
    "M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolveSecretarySourceRouteRequest {
    pub(crate) source_route_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4SourceRouteResolution {
    pub(crate) schema_version: String,
    pub(crate) source_owner_ref: String,
    pub(crate) source_object_type: String,
    pub(crate) canonical_source_object_id: String,
    /// Canonical decimal string; it is never converted through a JS number.
    pub(crate) source_revision: String,
    pub(crate) source_route_ref: String,
    pub(crate) target: M4SourceNavigationTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub(crate) enum M4SourceNavigationTarget {
    #[serde(rename = "WORK_ITEM")]
    WorkItem {
        project_id: String,
        workflow_id: String,
        work_item_id: String,
        source_revision: String,
    },
    #[serde(rename = "CONSULTATION_PROPOSAL")]
    ConsultationProposal {
        project_id: String,
        workflow_id: String,
        proposal_id: String,
        source_revision: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegisteredOwnerKind {
    WorkItem,
    ConsultationProposal,
}

/// Runtime registration is intentionally absent.  These are the only two
/// native source owners admitted by the frozen M4 remediation contract.
#[derive(Clone, Debug)]
pub(crate) struct M4RegisteredSourceOwnerRouteRegistry {
    workflow_state_path: PathBuf,
    m4_repository: M4SecretarySqliteRepository,
    #[cfg(test)]
    owner_repository_for_tests:
        Option<crate::workbench_sqlite_repository::WorkbenchSqliteRepository>,
}

impl M4RegisteredSourceOwnerRouteRegistry {
    pub(crate) fn new(
        workflow_state_path: &Path,
        m4_repository: M4SecretarySqliteRepository,
    ) -> Self {
        Self {
            workflow_state_path: workflow_state_path.to_path_buf(),
            m4_repository,
            #[cfg(test)]
            owner_repository_for_tests: None,
        }
    }

    #[cfg(test)]
    fn new_with_owner_repository_for_tests(
        m4_repository: M4SecretarySqliteRepository,
        owner_repository: crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
    ) -> Self {
        Self {
            workflow_state_path: PathBuf::new(),
            m4_repository,
            owner_repository_for_tests: Some(owner_repository),
        }
    }

    pub(crate) fn resolve(
        &self,
        request: &ResolveSecretarySourceRouteRequest,
    ) -> Result<M4SourceRouteResolution, String> {
        validate_source_route_ref(&request.source_route_ref)?;
        let candidate = self
            .m4_repository
            .read_current_source_route_candidate(&request.source_route_ref)
            .map_err(map_m4_candidate_error)?;
        if candidate.scope_ref != m4_primary_scope_ref() {
            return Err(M4_SOURCE_SCOPE_MISMATCH.to_string());
        }
        if candidate.source_type != M4_WORKFLOW_ATTENTION_SOURCE_TYPE {
            return Err(M4_SOURCE_ROUTE_TAMPERED.to_string());
        }
        let owner_kind = registered_owner_kind(&candidate)?;
        validate_route_seal(&candidate)?;

        let owner_repository = self.owner_repository()?;
        let expected = M4SourceOwnerPublicationExpectationV1 {
            publication_sequence: candidate.publication_sequence,
            publication_id: candidate.publication_id.clone(),
            adapter_id: candidate.adapter_id.clone(),
            publication_kind: candidate.publication_kind.clone(),
            source_owner_ref: candidate.source_owner_ref.clone(),
            object_type: candidate.source_object_type.clone(),
            canonical_object_id: candidate.canonical_source_object_id.clone(),
            source_revision: candidate.source_revision,
            source_event_id: candidate.source_event_id.clone(),
            source_owner_watermark: candidate.source_owner_watermark.clone(),
            native_scope_seal: candidate.native_scope_seal.clone(),
            opaque_route_ref: candidate.source_route_ref.clone(),
            payload_hash: candidate.payload_hash.clone(),
            m4_ingestion_receipt_id: candidate.m4_ingestion_receipt_id.clone(),
        };
        let target = owner_repository
            .validate_current_source_revision_and_target(&expected)
            .map_err(map_owner_resolution_error)?;
        let source_revision = candidate.source_revision.to_string();
        let target = match (owner_kind, target) {
            (
                RegisteredOwnerKind::WorkItem,
                M4SourceOwnerNavigationTargetRead::WorkItem {
                    project_id,
                    workflow_id,
                    work_item_id,
                },
            ) if work_item_id == candidate.canonical_source_object_id => {
                M4SourceNavigationTarget::WorkItem {
                    project_id,
                    workflow_id,
                    work_item_id,
                    source_revision: source_revision.clone(),
                }
            }
            (
                RegisteredOwnerKind::ConsultationProposal,
                M4SourceOwnerNavigationTargetRead::ConsultationProposal {
                    project_id,
                    workflow_id,
                    proposal_id,
                },
            ) if proposal_id == candidate.canonical_source_object_id => {
                M4SourceNavigationTarget::ConsultationProposal {
                    project_id,
                    workflow_id,
                    proposal_id,
                    source_revision: source_revision.clone(),
                }
            }
            _ => return Err(M4_SOURCE_TARGET_INTEGRITY_FAILED.to_string()),
        };
        Ok(M4SourceRouteResolution {
            schema_version: M4_SOURCE_ROUTE_RESOLUTION_SCHEMA.to_string(),
            source_owner_ref: candidate.source_owner_ref,
            source_object_type: candidate.source_object_type,
            canonical_source_object_id: candidate.canonical_source_object_id,
            source_revision,
            source_route_ref: candidate.source_route_ref,
            target,
        })
    }

    fn owner_repository(
        &self,
    ) -> Result<crate::workbench_sqlite_repository::WorkbenchSqliteRepository, String> {
        #[cfg(test)]
        if let Some(repository) = self.owner_repository_for_tests.as_ref() {
            return Ok(repository.clone());
        }
        crate::workbench_sqlite_storage_mode::primary_repository_for_m4_source_route_read(
            &self.workflow_state_path,
        )
        .map_err(|_| M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE.to_string())?
        .ok_or_else(|| M4_SOURCE_ROUTE_REGISTRY_UNAVAILABLE.to_string())
    }
}

fn validate_source_route_ref(value: &str) -> Result<(), String> {
    const PREFIX: &str = "source-route:sha256:";
    if value.len() != PREFIX.len() + 64
        || !value.starts_with(PREFIX)
        || !value[PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(M4_SOURCE_ROUTE_INVALID.to_string());
    }
    Ok(())
}

fn registered_owner_kind(
    candidate: &M4CurrentSourceRouteCandidate,
) -> Result<RegisteredOwnerKind, String> {
    let expected = if candidate.source_owner_ref == M4_WORK_ITEM_SOURCE_OWNER_REF {
        (
            RegisteredOwnerKind::WorkItem,
            "workflow_attention",
            M4_WORK_ITEM_SOURCE_ADAPTER_ID,
            "WORK_ITEM_ATTENTION",
        )
    } else if candidate.source_owner_ref == M4_PROPOSAL_SOURCE_OWNER_REF {
        (
            RegisteredOwnerKind::ConsultationProposal,
            "proposal_decision",
            M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
            "PROPOSAL_DECISION",
        )
    } else {
        return Err(M4_SOURCE_OWNER_UNREGISTERED.to_string());
    };
    if candidate.source_object_type != expected.1 {
        return Err(M4_SOURCE_TYPE_UNREGISTERED.to_string());
    }
    if candidate.adapter_id != expected.2 || candidate.publication_kind != expected.3 {
        return Err(M4_SOURCE_ROUTE_TAMPERED.to_string());
    }
    if !is_safe_target_identifier(&candidate.canonical_source_object_id) {
        return Err(M4_SOURCE_ROUTE_TAMPERED.to_string());
    }
    Ok(expected.0)
}

fn is_safe_target_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.contains('/')
        && !value.contains('\\')
        && !value.to_ascii_lowercase().contains("://")
}

fn validate_route_seal(candidate: &M4CurrentSourceRouteCandidate) -> Result<(), String> {
    let revision = candidate.source_revision.to_string();
    let expected = m4_internal_id(
        "source-route:sha256:",
        "syn.m4.registered-owner-route/v1",
        &[
            &candidate.source_owner_ref,
            &candidate.source_object_type,
            &candidate.canonical_source_object_id,
            &revision,
            &candidate.native_scope_seal,
        ],
    )
    .map_err(|_| M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE.to_string())?;
    if expected != candidate.source_route_ref {
        return Err(M4_SOURCE_ROUTE_TAMPERED.to_string());
    }
    Ok(())
}

fn map_m4_candidate_error(error: M4SecretaryRepositoryError) -> String {
    match error.code.as_str() {
        "m4_source_route_invalid" => M4_SOURCE_ROUTE_INVALID,
        "m4_source_route_stale" => M4_SOURCE_ROUTE_STALE,
        "m4_source_route_not_found"
        | "m4_source_route_ambiguous"
        | "m4_source_route_provenance_invalid" => M4_SOURCE_ROUTE_TAMPERED,
        _ => M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE,
    }
    .to_string()
}

fn map_owner_resolution_error(error: String) -> String {
    match error.as_str() {
        "m4_source_route_owner_revision_mismatch" => M4_SOURCE_REVISION_MISMATCH,
        "m4_source_route_owner_scope_mismatch" => M4_SOURCE_SCOPE_MISMATCH,
        "m4_source_route_target_missing" => M4_SOURCE_TARGET_MISSING,
        "m4_source_route_owner_unregistered" => M4_SOURCE_OWNER_UNREGISTERED,
        "m4_source_route_target_integrity_failed"
        | "m4_source_route_owner_publication_missing"
        | "m4_source_route_owner_terminal_receipt_mismatch"
        | "m4_source_route_owner_publication_invalid"
        | "m4_source_route_owner_publication_mismatch" => M4_SOURCE_TARGET_INTEGRITY_FAILED,
        _ => M4_SOURCE_ROUTE_RESOLUTION_UNAVAILABLE,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m4_secretary_repository::M4_ORDINARY_SECRETARY_RELATIVE_PATH;
    use crate::m4_source_owner_schema::{
        append_m4_proposal_source_publication, append_m4_work_item_source_publication,
        build_m4_proposal_source_publication, build_m4_work_item_source_publication,
        M4SourceOwnerOutboxEnvelopeV1,
    };
    use crate::utils::hash::sha256_hex;
    use crate::workbench_sqlite_repository::WorkbenchSqliteRepository;
    use rusqlite::{params, Connection};
    use serde_json::{json, Value};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    const PROJECT_ID: &str = "project:route-fixture";
    const WORKFLOW_ID: &str = "workflow:route-fixture";
    const NODE_ID: &str = "node:route-fixture";
    const SHARED_OBJECT_ID: &str = "object:shared-owner-collision";
    const WORK_EVENT_ID: &str = "event:route-work-item:7";
    const WORK_RECEIPT_ID: &str = "receipt:route-work-item:7";
    const WORK_SOURCE_REF: &str = "workflow_state:project:route-fixture:workflow:route-fixture";
    const PROPOSAL_AUDIT_ID: &str = "audit:route-proposal:9";
    static FULL_FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct FullRegistryFixture {
        root: PathBuf,
        owner_db_path: PathBuf,
        m4_db_path: PathBuf,
        owner_repository: WorkbenchSqliteRepository,
        m4_repository: M4SecretarySqliteRepository,
        registry: M4RegisteredSourceOwnerRouteRegistry,
        work_publication: M4SourceOwnerOutboxEnvelopeV1,
        proposal_publication: M4SourceOwnerOutboxEnvelopeV1,
    }

    impl FullRegistryFixture {
        fn new() -> Self {
            let sequence = FULL_FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let requested = std::env::temp_dir().join(format!(
                "syn-m4c03-route-resolver-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&requested).expect("create full route fixture root");
            let root = fs::canonicalize(requested).expect("canonical full route fixture root");
            let owner_db_path = root.join("owner-workbench.sqlite3");
            let owner_repository = WorkbenchSqliteRepository::open_rehearsal(&owner_db_path)
                .expect("open owner fixture repository");
            let m4_repository = M4SecretarySqliteRepository::open_isolated_fixture(&root)
                .expect("open M4 fixture repository");
            m4_repository
                .set_test_server_utc_now("2026-08-10T12:00:00.000Z")
                .expect("fix M4 server clock");
            let m4_db_path = root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH);
            let (work_publication, proposal_publication) = seed_owner_publications(&owner_db_path);
            let dispatched = crate::m4_source_dispatcher::dispatch_pending_m4_source_owner_outbox(
                &owner_repository,
                &m4_repository,
                "route-resolver-integration",
                8,
            )
            .expect("dispatch real owner publications into M4");
            assert_eq!(dispatched.delivered_count, 2);
            assert_eq!(dispatched.quarantined_count, 0);
            let registry =
                M4RegisteredSourceOwnerRouteRegistry::new_with_owner_repository_for_tests(
                    m4_repository.clone(),
                    owner_repository.clone(),
                );
            Self {
                root,
                owner_db_path,
                m4_db_path,
                owner_repository,
                m4_repository,
                registry,
                work_publication,
                proposal_publication,
            }
        }

        fn resolve(&self, route: &str) -> Result<M4SourceRouteResolution, String> {
            self.registry.resolve(&ResolveSecretarySourceRouteRequest {
                source_route_ref: route.to_string(),
            })
        }

        fn owner_connection(&self) -> Connection {
            let connection = Connection::open(&self.owner_db_path).expect("open owner fixture DB");
            connection
                .pragma_update(None, "foreign_keys", "ON")
                .expect("enable owner fixture foreign keys");
            connection
        }

        fn m4_connection(&self) -> Connection {
            let connection = Connection::open(&self.m4_db_path).expect("open M4 fixture DB");
            connection
                .pragma_update(None, "foreign_keys", "ON")
                .expect("enable M4 fixture foreign keys");
            connection
        }

        fn advance_proposal_owner_only(&self) -> String {
            let proposal = json!({
                "proposal_id": SHARED_OBJECT_ID,
                "project_id": PROJECT_ID,
                "workflow_id": WORKFLOW_ID,
                "status": "user_confirmed"
            });
            let audit = json!({
                "audit_event_id": "audit:route-proposal:10",
                "project_id": PROJECT_ID,
                "workflow_id": WORKFLOW_ID,
                "proposal_id": SHARED_OBJECT_ID,
                "after_status": "user_confirmed",
                "created_at_ms": 1_785_000_010_000_i64
            });
            let (proposal_hash, proposal_json) = hashed_record(&proposal);
            let (audit_hash, audit_json) = hashed_record(&audit);
            let mut connection = self.owner_connection();
            let transaction = connection.transaction().expect("begin proposal advance");
            transaction
                .execute(
                    "UPDATE project_proposals
                     SET record_hash = ?1, record_json = ?2
                     WHERE proposal_id = ?3",
                    params![proposal_hash, proposal_json, SHARED_OBJECT_ID],
                )
                .expect("advance proposal owner record");
            transaction
                .execute(
                    "INSERT INTO workflow_audit_events
                     (event_id, target_kind, target_id, source_id, record_hash, record_json)
                     VALUES (?1, 'project_consultation_proposal', ?2, 'route-fixture', ?3, ?4)",
                    params![
                        "audit:route-proposal:10",
                        SHARED_OBJECT_ID,
                        audit_hash,
                        audit_json
                    ],
                )
                .expect("insert proposal advance audit");
            let publication = build_m4_proposal_source_publication(
                &transaction,
                SHARED_OBJECT_ID,
                "audit:route-proposal:10",
                10,
            )
            .expect("build advanced proposal publication");
            append_m4_proposal_source_publication(&transaction, &publication)
                .expect("append advanced proposal publication");
            transaction.commit().expect("commit proposal advance");
            publication.opaque_route_ref
        }

        fn dispatch_owner_publications(&self) {
            let dispatched = crate::m4_source_dispatcher::dispatch_pending_m4_source_owner_outbox(
                &self.owner_repository,
                &self.m4_repository,
                "route-resolver-integration-advance",
                8,
            )
            .expect("dispatch advanced proposal publication");
            assert_eq!(dispatched.delivered_count, 1);
        }
    }

    impl Drop for FullRegistryFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn hashed_record(value: &Value) -> (String, String) {
        let record_json = serde_json::to_string(value).expect("serialize fixture record");
        (sha256_hex(&record_json), record_json)
    }

    fn seed_owner_publications(
        owner_db_path: &Path,
    ) -> (M4SourceOwnerOutboxEnvelopeV1, M4SourceOwnerOutboxEnvelopeV1) {
        let project = json!({ "project_id": PROJECT_ID });
        let workflow = json!({ "workflow_id": WORKFLOW_ID, "project_id": PROJECT_ID });
        let node = json!({ "node_id": NODE_ID, "workflow_id": WORKFLOW_ID });
        let work_item = json!({
            "work_item_id": SHARED_OBJECT_ID,
            "workflow_id": WORKFLOW_ID,
            "current_node_id": NODE_ID,
            "state": "running",
            "workflow_revision_after": 7
        });
        let proposal = json!({
            "proposal_id": SHARED_OBJECT_ID,
            "project_id": PROJECT_ID,
            "workflow_id": WORKFLOW_ID,
            "status": "pending_user_confirmation"
        });
        let proposal_audit = json!({
            "audit_event_id": PROPOSAL_AUDIT_ID,
            "project_id": PROJECT_ID,
            "workflow_id": WORKFLOW_ID,
            "proposal_id": SHARED_OBJECT_ID,
            "after_status": "pending_user_confirmation",
            "created_at_ms": 1_785_000_000_000_i64
        });
        let (project_hash, project_json) = hashed_record(&project);
        let (workflow_hash, workflow_json) = hashed_record(&workflow);
        let (node_hash, node_json) = hashed_record(&node);
        let (work_hash, work_json) = hashed_record(&work_item);
        let (proposal_hash, proposal_json) = hashed_record(&proposal);
        let (audit_hash, audit_json) = hashed_record(&proposal_audit);
        let status_hash = sha256_hex("running");

        let mut connection = Connection::open(owner_db_path).expect("open owner DB for seed");
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .expect("enable owner seed foreign keys");
        let transaction = connection.transaction().expect("begin owner route seed");
        transaction
            .execute(
                "INSERT INTO projects (project_id, source_id, record_hash, record_json)
                 VALUES (?1, 'route-fixture', ?2, ?3)",
                params![PROJECT_ID, project_hash, project_json],
            )
            .expect("insert route project");
        transaction
            .execute(
                "INSERT INTO workflows
                 (workflow_id, project_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, 'route-fixture', ?3, ?4)",
                params![WORKFLOW_ID, PROJECT_ID, workflow_hash, workflow_json],
            )
            .expect("insert route workflow");
        transaction
            .execute(
                "INSERT INTO workflow_nodes
                 (node_id, workflow_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, 'route-fixture', ?3, ?4)",
                params![NODE_ID, WORKFLOW_ID, node_hash, node_json],
            )
            .expect("insert route node");
        transaction
            .execute(
                "INSERT INTO work_items
                 (work_item_id, workflow_id, node_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, ?3, 'route-fixture', ?4, ?5)",
                params![
                    SHARED_OBJECT_ID,
                    WORKFLOW_ID,
                    Option::<String>::None,
                    work_hash,
                    work_json
                ],
            )
            .expect("insert legacy current-node-only route work item");
        transaction
            .execute(
                "INSERT INTO project_proposals
                 (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, ?3, 'route-fixture', ?4, ?5)",
                params![
                    SHARED_OBJECT_ID,
                    PROJECT_ID,
                    WORKFLOW_ID,
                    proposal_hash,
                    proposal_json
                ],
            )
            .expect("insert route proposal");
        transaction
            .execute(
                "INSERT INTO workflow_audit_events
                 (event_id, target_kind, target_id, source_id, record_hash, record_json)
                 VALUES (?1, 'project_consultation_proposal', ?2, 'route-fixture', ?3, ?4)",
                params![PROPOSAL_AUDIT_ID, SHARED_OBJECT_ID, audit_hash, audit_json],
            )
            .expect("insert route proposal audit");
        transaction
            .execute(
                "INSERT INTO commands (command_id, registered_at)
                 VALUES ('command:route-work-item:7', '2026-08-10T10:00:00.000Z')",
                [],
            )
            .expect("insert route owner command");
        transaction
            .execute(
                "INSERT INTO command_receipts (
                    receipt_id, command_id, idempotency_key, request_hash,
                    actor_id, scope_ref, policy_decision_ref, status,
                    accepted_at, result_ref, result_hash, committed_revision, created_at
                 ) VALUES (
                    ?1, 'command:route-work-item:7', 'idem:route-work-item:7', ?2,
                    'user', 'scope:route-fixture', 'policy:allowed', 'COMMITTED',
                    '2026-08-10T10:00:00.000Z', 'result:route-work-item:7', ?3, 7,
                    '2026-08-10T10:00:00.000Z'
                 )",
                params![WORK_RECEIPT_ID, "f".repeat(64), status_hash],
            )
            .expect("insert route owner receipt");
        transaction
            .execute(
                "INSERT INTO events (
                    event_id, event_type, occurred_at, actor_id, scope_ref,
                    source_ref, source_revision, command_id, schema_version,
                    sensitivity, payload_hash, created_at
                 ) VALUES (
                    ?1, 'WorkItemStateUpdated', '2026-08-10T10:00:00.000Z', 'user',
                    'scope:route-fixture', ?2, '7', 'command:route-work-item:7',
                    '1.0.0', 'INTERNAL', ?3, '2026-08-10T10:00:00.000Z'
                 )",
                params![WORK_EVENT_ID, WORK_SOURCE_REF, status_hash],
            )
            .expect("insert route owner event");
        transaction
            .execute(
                "INSERT INTO projectors (projector_id, projector_version, registered_at)
                 VALUES ('workflow_projector', 'route-fixture.v1',
                         '2026-08-10T10:00:00.000Z')",
                [],
            )
            .expect("insert route owner projector");
        transaction
            .execute(
                "INSERT INTO current_snapshots (
                    object_ref, object_revision, source_watermark,
                    snapshot_hash, projector_id, built_at
                 ) VALUES (
                    ?1, 7, ?2, ?3, 'workflow_projector',
                    '2026-08-10T10:00:00.000Z'
                 )",
                params![WORK_SOURCE_REF, WORK_EVENT_ID, "a".repeat(64)],
            )
            .expect("insert route owner snapshot");

        let work_publication = build_m4_work_item_source_publication(
            &transaction,
            WORK_EVENT_ID,
            WORK_RECEIPT_ID,
            SHARED_OBJECT_ID,
            "running",
        )
        .expect("build real WorkItem publication");
        assert_eq!(
            append_m4_work_item_source_publication(&transaction, &work_publication)
                .expect("append real WorkItem publication"),
            1
        );
        let proposal_publication = build_m4_proposal_source_publication(
            &transaction,
            SHARED_OBJECT_ID,
            PROPOSAL_AUDIT_ID,
            9,
        )
        .expect("build real proposal publication");
        assert_eq!(
            append_m4_proposal_source_publication(&transaction, &proposal_publication)
                .expect("append real proposal publication"),
            2
        );
        transaction
            .commit()
            .expect("commit real owner publications");
        (work_publication, proposal_publication)
    }

    fn digest(value: char) -> String {
        format!("source-route:sha256:{}", value.to_string().repeat(64))
    }

    fn candidate(
        owner: &str,
        object_type: &str,
        adapter: &str,
        kind: &str,
    ) -> M4CurrentSourceRouteCandidate {
        let canonical_source_object_id = "shared-object-id".to_string();
        let native_scope_seal = format!("native-scope:sha256:{}", "b".repeat(64));
        let source_revision = 7;
        let source_route_ref = m4_internal_id(
            "source-route:sha256:",
            "syn.m4.registered-owner-route/v1",
            &[
                owner,
                object_type,
                &canonical_source_object_id,
                &source_revision.to_string(),
                &native_scope_seal,
            ],
        )
        .expect("route");
        M4CurrentSourceRouteCandidate {
            source_event_key: "event-key".to_string(),
            source_identity_key: "identity-key".to_string(),
            source_owner_ref: owner.to_string(),
            scope_ref: m4_primary_scope_ref().to_string(),
            source_type: M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string(),
            canonical_source_object_id,
            source_revision,
            source_event_id: format!("source-event:sha256:{}", "c".repeat(64)),
            source_owner_watermark: format!("source-watermark:sha256:{}", "d".repeat(64)),
            source_route_ref,
            payload_hash: "e".repeat(64),
            publication_sequence: 1,
            publication_id: format!("source-publication:sha256:{}", "f".repeat(64)),
            adapter_id: adapter.to_string(),
            publication_kind: kind.to_string(),
            native_scope_seal,
            source_object_type: object_type.to_string(),
            m4_ingestion_receipt_id: "receipt".to_string(),
        }
    }

    #[test]
    fn full_registry_resolves_real_delivered_work_item_and_proposal_owner_collision() {
        let fixture = FullRegistryFixture::new();
        let work = fixture
            .resolve(&fixture.work_publication.opaque_route_ref)
            .expect("resolve delivered WorkItem route");
        assert_eq!(work.source_owner_ref, M4_WORK_ITEM_SOURCE_OWNER_REF);
        assert_eq!(work.source_object_type, "workflow_attention");
        assert_eq!(work.canonical_source_object_id, SHARED_OBJECT_ID);
        assert_eq!(work.source_revision, "7");
        assert_eq!(
            work.target,
            M4SourceNavigationTarget::WorkItem {
                project_id: PROJECT_ID.to_string(),
                workflow_id: WORKFLOW_ID.to_string(),
                work_item_id: SHARED_OBJECT_ID.to_string(),
                source_revision: "7".to_string(),
            }
        );

        let proposal = fixture
            .resolve(&fixture.proposal_publication.opaque_route_ref)
            .expect("resolve delivered proposal route");
        assert_eq!(proposal.source_owner_ref, M4_PROPOSAL_SOURCE_OWNER_REF);
        assert_eq!(proposal.source_object_type, "proposal_decision");
        assert_eq!(proposal.canonical_source_object_id, SHARED_OBJECT_ID);
        assert_eq!(proposal.source_revision, "9");
        assert_eq!(
            proposal.target,
            M4SourceNavigationTarget::ConsultationProposal {
                project_id: PROJECT_ID.to_string(),
                workflow_id: WORKFLOW_ID.to_string(),
                proposal_id: SHARED_OBJECT_ID.to_string(),
                source_revision: "9".to_string(),
            }
        );
        assert_ne!(work.source_route_ref, proposal.source_route_ref);
    }

    #[test]
    fn full_registry_returns_fixed_failures_for_stale_revision_missing_and_tamper() {
        let fixture = FullRegistryFixture::new();
        let work_route = fixture.work_publication.opaque_route_ref.clone();
        let proposal_route = fixture.proposal_publication.opaque_route_ref.clone();

        let owner = fixture.owner_connection();
        let work_terminal_receipt: String = owner
            .query_row(
                "SELECT terminal_receipt_ref FROM m4_source_owner_publications
                 WHERE publication_id = ?1",
                [fixture.work_publication.publication_id.as_str()],
                |row| row.get(0),
            )
            .expect("read exact WorkItem terminal receipt");
        owner
            .execute(
                "UPDATE m4_source_owner_publications
                 SET terminal_receipt_ref = 'ingestion-receipt:tampered'
                 WHERE publication_id = ?1",
                [fixture.work_publication.publication_id.as_str()],
            )
            .expect("tamper terminal receipt");
        assert_eq!(
            fixture.resolve(&work_route),
            Err(M4_SOURCE_TARGET_INTEGRITY_FAILED.to_string())
        );
        owner
            .execute(
                "UPDATE m4_source_owner_publications SET terminal_receipt_ref = ?1
                 WHERE publication_id = ?2",
                params![
                    work_terminal_receipt,
                    fixture.work_publication.publication_id
                ],
            )
            .expect("restore terminal receipt");

        owner
            .execute(
                "UPDATE current_snapshots
                 SET object_revision = 8, source_watermark = 'event:unrelated-work-item:8'
                 WHERE object_ref = ?1 AND projector_id = 'workflow_projector'",
                [WORK_SOURCE_REF],
            )
            .expect("advance owner snapshot through unrelated WorkItem");
        assert_eq!(
            fixture.resolve(&work_route),
            Err(M4_SOURCE_REVISION_MISMATCH.to_string())
        );
        owner
            .execute(
                "UPDATE current_snapshots
                 SET object_revision = 7, source_watermark = ?1
                 WHERE object_ref = ?2 AND projector_id = 'workflow_projector'",
                params![WORK_EVENT_ID, WORK_SOURCE_REF],
            )
            .expect("restore owner snapshot");

        let proposal_row: (String, String, String, String, String) = owner
            .query_row(
                "SELECT project_id, workflow_id, source_id, record_hash, record_json
                 FROM project_proposals WHERE proposal_id = ?1",
                [SHARED_OBJECT_ID],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .expect("read proposal before missing-record proof");
        owner
            .execute(
                "DELETE FROM project_proposals WHERE proposal_id = ?1",
                [SHARED_OBJECT_ID],
            )
            .expect("remove proposal owner record");
        assert_eq!(
            fixture.resolve(&proposal_route),
            Err(M4_SOURCE_TARGET_MISSING.to_string())
        );
        owner
            .execute(
                "INSERT INTO project_proposals
                 (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    SHARED_OBJECT_ID,
                    proposal_row.0,
                    proposal_row.1,
                    proposal_row.2,
                    proposal_row.3,
                    proposal_row.4,
                ],
            )
            .expect("restore proposal owner record");
        drop(owner);

        let m4 = fixture.m4_connection();
        let altered_scope = format!("native-scope:sha256:{}", "c".repeat(64));
        m4.execute(
            "UPDATE m4_source_provenance_index SET native_scope_seal = ?1
             WHERE publication_id = ?2",
            params![altered_scope, fixture.proposal_publication.publication_id],
        )
        .expect("tamper M4 route provenance");
        assert_eq!(
            fixture.resolve(&proposal_route),
            Err(M4_SOURCE_ROUTE_TAMPERED.to_string())
        );
        m4.execute(
            "UPDATE m4_source_provenance_index SET native_scope_seal = ?1
             WHERE publication_id = ?2",
            params![
                fixture.proposal_publication.native_scope_seal,
                fixture.proposal_publication.publication_id
            ],
        )
        .expect("restore M4 route provenance");

        m4.execute(
            "UPDATE m4_admitted_source_events SET source_owner_ref = 'owner:unknown:fixture'
             WHERE source_link_ref = ?1",
            [proposal_route.as_str()],
        )
        .expect("install unknown event owner");
        m4.execute(
            "UPDATE m4_admitted_source_current SET source_owner_ref = 'owner:unknown:fixture'
             WHERE source_link_ref = ?1",
            [proposal_route.as_str()],
        )
        .expect("install unknown current owner");
        assert_eq!(
            fixture.resolve(&proposal_route),
            Err(M4_SOURCE_OWNER_UNREGISTERED.to_string())
        );
        m4.execute(
            "UPDATE m4_admitted_source_events SET source_owner_ref = ?1
             WHERE source_link_ref = ?2",
            params![M4_PROPOSAL_SOURCE_OWNER_REF, proposal_route],
        )
        .expect("restore event owner");
        m4.execute(
            "UPDATE m4_admitted_source_current SET source_owner_ref = ?1
             WHERE source_link_ref = ?2",
            params![M4_PROPOSAL_SOURCE_OWNER_REF, proposal_route],
        )
        .expect("restore current owner");

        m4.execute(
            "UPDATE m4_source_provenance_index
             SET source_object_type = 'unregistered_native_type'
             WHERE publication_id = ?1",
            [fixture.proposal_publication.publication_id.as_str()],
        )
        .expect("install unknown native type");
        assert_eq!(
            fixture.resolve(&proposal_route),
            Err(M4_SOURCE_TYPE_UNREGISTERED.to_string())
        );
        m4.execute(
            "UPDATE m4_source_provenance_index SET source_object_type = 'proposal_decision'
             WHERE publication_id = ?1",
            [fixture.proposal_publication.publication_id.as_str()],
        )
        .expect("restore native type");

        let mismatched_scope = format!("native-scope:sha256:{}", "d".repeat(64));
        let mismatched_route = m4_internal_id(
            "source-route:sha256:",
            "syn.m4.registered-owner-route/v1",
            &[
                M4_PROPOSAL_SOURCE_OWNER_REF,
                "proposal_decision",
                SHARED_OBJECT_ID,
                "9",
                &mismatched_scope,
            ],
        )
        .expect("build scope-mismatched sealed route");
        m4.execute(
            "UPDATE m4_source_provenance_index SET native_scope_seal = ?1
             WHERE publication_id = ?2",
            params![
                mismatched_scope,
                fixture.proposal_publication.publication_id
            ],
        )
        .expect("install mismatched M4 scope");
        m4.execute(
            "UPDATE m4_admitted_source_events SET source_link_ref = ?1
             WHERE source_link_ref = ?2",
            params![mismatched_route, proposal_route],
        )
        .expect("install scope-mismatched historical route");
        m4.execute(
            "UPDATE m4_admitted_source_current SET source_link_ref = ?1
             WHERE source_link_ref = ?2",
            params![mismatched_route, proposal_route],
        )
        .expect("install scope-mismatched current route");
        assert_eq!(
            fixture.resolve(&mismatched_route),
            Err(M4_SOURCE_SCOPE_MISMATCH.to_string())
        );
        m4.execute(
            "UPDATE m4_admitted_source_events SET source_link_ref = ?1
             WHERE source_link_ref = ?2",
            params![proposal_route, mismatched_route],
        )
        .expect("restore historical route");
        m4.execute(
            "UPDATE m4_admitted_source_current SET source_link_ref = ?1
             WHERE source_link_ref = ?2",
            params![proposal_route, mismatched_route],
        )
        .expect("restore current route");
        m4.execute(
            "UPDATE m4_source_provenance_index SET native_scope_seal = ?1
             WHERE publication_id = ?2",
            params![
                fixture.proposal_publication.native_scope_seal,
                fixture.proposal_publication.publication_id
            ],
        )
        .expect("restore M4 native scope");
        drop(m4);

        let advanced_route = fixture.advance_proposal_owner_only();
        assert_eq!(
            fixture.resolve(&proposal_route),
            Err(M4_SOURCE_REVISION_MISMATCH.to_string())
        );
        fixture.dispatch_owner_publications();
        assert_eq!(
            fixture.resolve(&proposal_route),
            Err(M4_SOURCE_ROUTE_STALE.to_string())
        );
        let advanced = fixture
            .resolve(&advanced_route)
            .expect("advanced proposal route remains resolvable");
        assert_eq!(advanced.source_revision, "10");
    }

    #[test]
    fn request_is_route_only_and_rejects_extra_authority() {
        let accepted: ResolveSecretarySourceRouteRequest = serde_json::from_value(json!({
            "source_route_ref": digest('a')
        }))
        .expect("route-only request");
        assert!(validate_source_route_ref(&accepted.source_route_ref).is_ok());
        assert!(
            serde_json::from_value::<ResolveSecretarySourceRouteRequest>(json!({
                "source_route_ref": digest('a'),
                "source_owner_ref": M4_WORK_ITEM_SOURCE_OWNER_REF
            }))
            .is_err()
        );
        assert_eq!(
            validate_source_route_ref(&digest('A')),
            Err(M4_SOURCE_ROUTE_INVALID.to_string())
        );
    }

    #[test]
    fn closed_registry_keeps_same_object_id_on_distinct_owner_axes() {
        let work_item = candidate(
            M4_WORK_ITEM_SOURCE_OWNER_REF,
            "workflow_attention",
            M4_WORK_ITEM_SOURCE_ADAPTER_ID,
            "WORK_ITEM_ATTENTION",
        );
        let proposal = candidate(
            M4_PROPOSAL_SOURCE_OWNER_REF,
            "proposal_decision",
            M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
            "PROPOSAL_DECISION",
        );
        assert_eq!(
            registered_owner_kind(&work_item),
            Ok(RegisteredOwnerKind::WorkItem)
        );
        assert_eq!(
            registered_owner_kind(&proposal),
            Ok(RegisteredOwnerKind::ConsultationProposal)
        );
        assert_ne!(work_item.source_route_ref, proposal.source_route_ref);
        let mut tampered = work_item.clone();
        tampered.source_route_ref = digest('0');
        assert_eq!(
            validate_route_seal(&tampered),
            Err(M4_SOURCE_ROUTE_TAMPERED.to_string())
        );
    }

    #[test]
    fn success_wire_has_exact_keys_and_finite_target() {
        let response = M4SourceRouteResolution {
            schema_version: M4_SOURCE_ROUTE_RESOLUTION_SCHEMA.to_string(),
            source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF.to_string(),
            source_object_type: "workflow_attention".to_string(),
            canonical_source_object_id: "work-item:1".to_string(),
            source_revision: "18446744073709551615".to_string(),
            source_route_ref: digest('a'),
            target: M4SourceNavigationTarget::WorkItem {
                project_id: "project:1".to_string(),
                workflow_id: "workflow:1".to_string(),
                work_item_id: "work-item:1".to_string(),
                source_revision: "18446744073709551615".to_string(),
            },
        };
        let Value::Object(object) = serde_json::to_value(response).expect("serialize") else {
            panic!("object response")
        };
        assert_eq!(
            object
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            [
                "canonical_source_object_id",
                "schema_version",
                "source_object_type",
                "source_owner_ref",
                "source_revision",
                "source_route_ref",
                "target",
            ]
            .into_iter()
            .map(str::to_string)
            .collect()
        );
        let target = object
            .get("target")
            .and_then(Value::as_object)
            .expect("target");
        assert_eq!(
            target.get("kind"),
            Some(&Value::String("WORK_ITEM".to_string()))
        );
        assert!(
            !target.contains_key("path")
                && !target.contains_key("url")
                && !target.contains_key("view")
        );
    }
}
