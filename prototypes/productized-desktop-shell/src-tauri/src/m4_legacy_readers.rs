//! R06's closed, server-owned legacy reader registry.
//!
//! The registry observes only the historical server surfaces.  It never asks
//! the renderer for cache state, and it never derives a legacy tuple from the
//! new M4 Secretary brief.  Only the registered WorkItem owner publication
//! has enough independently-owned fields to construct a comparator candidate.

use crate::m4_secretary_domain::{
    m4_internal_id, m4_primary_scope_ref, m4_priority_reason, M4AttentionSignals,
    M4_WORKFLOW_ATTENTION_OBJECT_TYPE, M4_WORKFLOW_ATTENTION_SOURCE_TYPE,
};
use crate::m4_secretary_read_model::{
    M4LegacyReadCandidate, M4LegacyReadSourceKind, M4LegacyReaderReadState, M4LegacyReaderReceipt,
    M4LegacyServerOwnedReadBatch, M4SourceLinkRead, M4R06_EMPTY_SERVER_SURFACE,
    M4R06_READER_REJECTED, M4R06_READER_UNAVAILABLE, M4R06_UNJOINABLE_NO_EXACT_TUPLE,
};
use crate::m4_secretary_repository::M4SecretarySqliteRepository;
use crate::m4_source_owner_schema::{
    M4SourceOwnerOutboxEnvelopeV1, RegisteredWorkItemSourceOwnerMapper,
};
use crate::workbench_sqlite_repository::M4LegacyWorkItemSourceRead;
use std::path::{Path, PathBuf};

pub(crate) const M4R06_WORKFLOW_SNAPSHOT_REJECTED: &str = "m4r06_workflow_snapshot_not_initialized";

/// Ordinary composition installs this registry once.  It owns no write port;
/// every operation below is a fixed, query-only read.
#[derive(Clone, Debug)]
pub(crate) struct M4LegacyReadRegistry {
    index_path: PathBuf,
    workflow_state_path: PathBuf,
    m4_repository: M4SecretarySqliteRepository,
    work_item_reader: WorkItemLegacyShadowReader,
}

impl M4LegacyReadRegistry {
    pub(crate) fn new(
        index_path: &Path,
        workflow_state_path: &Path,
        m4_repository: M4SecretarySqliteRepository,
    ) -> Self {
        Self {
            index_path: index_path.to_path_buf(),
            workflow_state_path: workflow_state_path.to_path_buf(),
            m4_repository,
            work_item_reader: WorkItemLegacyShadowReader::new(workflow_state_path),
        }
    }

    /// Read all five fixed legacy surfaces in wire order.
    ///
    /// The M4 scope watermark is captured before the owner-db query.  The
    /// repository's later canonical reread must match it for a WorkItem row to
    /// be `PARITY`; a concurrent canonical update therefore remains a normal
    /// comparator quarantine rather than a stale fallback result.
    pub(crate) fn read_server_owned_legacy_candidates(
        &self,
    ) -> Result<M4LegacyServerOwnedReadBatch, String> {
        let server_read_timestamp = crate::unix_timestamp_string();
        let preliminary_scope_watermark = self
            .m4_repository
            .read_attention_snapshot(m4_primary_scope_ref())
            .map_err(|_| "m4r06_initial_scope_watermark_unavailable".to_string())?
            .scope_source_watermark;

        let secretary = self.read_secretary_summary_primitives();
        let (right_rail, candidates) =
            self.read_right_rail_work_items(&preliminary_scope_watermark);
        let runtime = self.read_runtime_attention_primitives(&server_read_timestamp);
        let react_pending = M4LegacyReaderReceipt::new(
            M4LegacyReadSourceKind::ReactPendingActionVisibility,
            M4LegacyReaderReadState::Unjoinable,
            Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE),
            None,
            0,
            0,
        );
        let memory = self.read_memory_daily_candidates(&server_read_timestamp);

        Ok(M4LegacyServerOwnedReadBatch {
            reader_receipts: vec![secretary, right_rail, runtime, react_pending, memory],
            candidates,
        })
    }

    /// Close the cross-database WorkItem cut after the M4 repository has
    /// produced its final canonical reread.  The first owner read and that
    /// canonical reread are independent SQLite transactions; an owner update
    /// that has not reached M4 must therefore make the whole report
    /// unavailable rather than leave an old candidate eligible for `PARITY`.
    ///
    /// This deliberately re-reads *only* the WorkItem surface.  Runtime and
    /// memory readers contain time-derived observations and are not part of
    /// the WorkItem owner consistency cut.
    pub(crate) fn verify_work_item_owner_cut(
        &self,
        initial_read: &M4LegacyServerOwnedReadBatch,
    ) -> Result<(), String> {
        let preliminary_scope_watermark = preliminary_work_item_owner_cut_watermark(initial_read)?;
        let (final_receipt, final_candidates) =
            self.read_right_rail_work_items(&preliminary_scope_watermark);
        verify_work_item_owner_cut_against(initial_read, &final_receipt, &final_candidates)
    }

    fn read_secretary_summary_primitives(&self) -> M4LegacyReaderReceipt {
        let kind = M4LegacyReadSourceKind::SecretaryReadModelDeterministicSummary;
        if self.workflow_state_path.exists() && !self.workflow_state_path.is_file() {
            return quarantined_receipt(kind, 0, "m4r06_secretary_surface_unavailable");
        }
        match read_initialized_legacy_workflow_snapshot(&self.workflow_state_path) {
            Ok(snapshot) if snapshot.project_workflows.is_empty() => empty_receipt(kind),
            Ok(snapshot) => unjoinable_receipt(kind, snapshot.project_workflows.len() as u64),
            Err(error) => quarantined_receipt(kind, 0, &error),
        }
    }

    fn read_right_rail_work_items(
        &self,
        preliminary_scope_watermark: &str,
    ) -> (M4LegacyReaderReceipt, Vec<M4LegacyReadCandidate>) {
        let kind = M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection;
        match self.work_item_reader.read_server_owned() {
            Ok(M4LegacyWorkItemSourceRead::Empty) => (empty_receipt(kind), Vec::new()),
            Ok(M4LegacyWorkItemSourceRead::Rejected { candidate_count }) => (
                M4LegacyReaderReceipt::new(
                    kind,
                    M4LegacyReaderReadState::Quarantined,
                    Some(M4R06_READER_REJECTED),
                    None,
                    candidate_count,
                    0,
                ),
                Vec::new(),
            ),
            Ok(M4LegacyWorkItemSourceRead::Observed(publications)) => {
                let candidate_count = publications.len() as u64;
                let candidates = publications
                    .iter()
                    .map(|publication| {
                        work_item_candidate(publication, preliminary_scope_watermark)
                    })
                    .collect::<Result<Vec<_>, _>>();
                match candidates {
                    Ok(candidates) => (
                        M4LegacyReaderReceipt::new(
                            kind,
                            M4LegacyReaderReadState::Observed,
                            None,
                            Some(RegisteredWorkItemSourceOwnerMapper::ADAPTER_ID),
                            candidate_count,
                            candidate_count,
                        ),
                        candidates,
                    ),
                    Err(_) => (
                        M4LegacyReaderReceipt::new(
                            kind,
                            M4LegacyReaderReadState::Quarantined,
                            Some(M4R06_READER_REJECTED),
                            None,
                            candidate_count,
                            0,
                        ),
                        Vec::new(),
                    ),
                }
            }
            Err(error) => (quarantined_receipt(kind, 0, &error), Vec::new()),
        }
    }

    fn read_runtime_attention_primitives(
        &self,
        server_read_timestamp: &str,
    ) -> M4LegacyReaderReceipt {
        let kind = M4LegacyReadSourceKind::RuntimeAttentionProjection;
        if (self.index_path.exists() && !self.index_path.is_file())
            || (self.workflow_state_path.exists() && !self.workflow_state_path.is_file())
        {
            return quarantined_receipt(kind, 0, "m4r06_runtime_surface_unavailable");
        }
        let continuation_sidecar =
            match crate::session_continuation_store::sidecar_path(&self.workflow_state_path) {
                Ok(path) => path,
                Err(error) => return quarantined_receipt(kind, 0, &error),
            };
        if continuation_sidecar.exists() && !continuation_sidecar.is_file() {
            return quarantined_receipt(kind, 0, "m4r06_runtime_surface_unavailable");
        }
        match crate::derive_runtime_session_attention_for_legacy_reader(
            &self.index_path,
            &self.workflow_state_path,
            server_read_timestamp,
        ) {
            Ok(attention) => {
                // The ordinary snapshot also emits generic, unbound protocol
                // descriptors when no runtime owner has supplied a binding.
                // Those descriptors are useful UI guard text, but are not a
                // server-owned legacy runtime surface.  Only a bound preview
                // or a persisted continuation/attempt is an observed legacy
                // runtime item; no such item is a genuine empty read.
                let legacy_surface_count = attention
                    .iter()
                    .filter(|item| runtime_attention_has_server_owned_legacy_surface(item))
                    .count() as u64;
                if legacy_surface_count == 0 {
                    empty_receipt(kind)
                } else {
                    unjoinable_receipt(kind, legacy_surface_count)
                }
            }
            Err(error) => quarantined_receipt(kind, 0, &error),
        }
    }

    fn read_memory_daily_candidates(&self, server_read_timestamp: &str) -> M4LegacyReaderReceipt {
        let kind = M4LegacyReadSourceKind::MemoryDailyInboxCandidate;
        let sidecar = match crate::memory_candidate_store::sidecar_path(&self.workflow_state_path) {
            Ok(path) => path,
            Err(error) => return quarantined_receipt(kind, 0, &error),
        };
        if sidecar.exists() && !sidecar.is_file() {
            return quarantined_receipt(kind, 0, "m4r06_memory_surface_unavailable");
        }
        match crate::memory_candidate_store::load_store(
            &self.workflow_state_path,
            server_read_timestamp,
        ) {
            Ok(store) if store.candidates.is_empty() => empty_receipt(kind),
            Ok(store) => unjoinable_receipt(kind, store.candidates.len() as u64),
            Err(error) => quarantined_receipt(kind, 0, &error),
        }
    }
}

fn read_initialized_legacy_workflow_snapshot(
    workflow_state_path: &Path,
) -> Result<crate::WorkflowStateSnapshot, String> {
    let snapshot = crate::read_workflow_state_snapshot(workflow_state_path)?;
    // The ordinary read model intentionally represents an absent optional
    // workflow-state file as an empty, uninitialized snapshot.  R06 must
    // preserve that server-surface EMPTY outcome, while rejecting an existing
    // JSON document whose schema validation failed.
    if snapshot.exists && !snapshot.initialized {
        return Err(M4R06_WORKFLOW_SNAPSHOT_REJECTED.to_string());
    }
    Ok(snapshot)
}

fn runtime_attention_has_server_owned_legacy_surface(
    attention: &crate::RuntimeSessionAttention,
) -> bool {
    let has_bound_runtime_identity = [
        attention.project_id.as_deref(),
        attention.workflow_id.as_deref(),
        attention.node_id.as_deref(),
        attention.session_id.as_deref(),
    ]
    .into_iter()
    .all(|value| value.is_some_and(|value| !value.trim().is_empty()));
    has_bound_runtime_identity
        && attention.source_refs.iter().any(|source| {
            matches!(
                source.source_kind.as_str(),
                "session_continuation_preview"
                    | "controlled_session_continuation"
                    | "session_continuation_attempt"
            )
        })
}

/// The sole exact-tuple reader.  Its name is intentionally explicit: this is
/// a legacy shadow adapter over the registered WorkItem owner, not a generic
/// outbox browser and not a renderer projection reader.
#[derive(Clone, Debug)]
struct WorkItemLegacyShadowReader {
    workflow_state_path: PathBuf,
}

impl WorkItemLegacyShadowReader {
    fn new(workflow_state_path: &Path) -> Self {
        Self {
            workflow_state_path: workflow_state_path.to_path_buf(),
        }
    }

    fn read_server_owned(&self) -> Result<M4LegacyWorkItemSourceRead, String> {
        let owner_repository =
            crate::workbench_sqlite_storage_mode::primary_repository_for_m4_source_route_read(
                &self.workflow_state_path,
            )?
            .ok_or_else(|| "m4r06_work_item_owner_repository_unavailable".to_string())?;
        owner_repository.read_current_delivered_work_item_legacy_sources()
    }
}

fn empty_receipt(kind: M4LegacyReadSourceKind) -> M4LegacyReaderReceipt {
    M4LegacyReaderReceipt::new(
        kind,
        M4LegacyReaderReadState::Empty,
        Some(M4R06_EMPTY_SERVER_SURFACE),
        None,
        0,
        0,
    )
}

fn unjoinable_receipt(kind: M4LegacyReadSourceKind, candidate_count: u64) -> M4LegacyReaderReceipt {
    M4LegacyReaderReceipt::new(
        kind,
        M4LegacyReaderReadState::Unjoinable,
        Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE),
        None,
        candidate_count,
        0,
    )
}

fn quarantined_receipt(
    kind: M4LegacyReadSourceKind,
    candidate_count: u64,
    error: &str,
) -> M4LegacyReaderReceipt {
    let reason_code = if legacy_reader_error_is_rejected(error) {
        M4R06_READER_REJECTED
    } else {
        M4R06_READER_UNAVAILABLE
    };
    M4LegacyReaderReceipt::new(
        kind,
        M4LegacyReaderReadState::Quarantined,
        Some(reason_code),
        None,
        candidate_count,
        0,
    )
}

/// The owner loaders deliberately distinguish malformed/invalid persisted
/// data from an unavailable read.  Match only their stable error prefixes:
/// filesystem paths can themselves contain words such as `invalid`, `schema`,
/// or `.json`, so substring classification would corrupt the receipt.
fn legacy_reader_error_is_rejected(error: &str) -> bool {
    const REJECTED_OWNER_ERROR_PREFIXES: &[&str] = &[
        M4R06_WORKFLOW_SNAPSHOT_REJECTED,
        "工作流状态 JSON 解析失败 ",
        "索引 JSON 解析失败 ",
        "continuation sidecar JSON 损坏，已拒绝覆盖 ",
        "continuation schema_version 不匹配：",
        "continuation store_version 不匹配：",
        "continuation storage_kind 不匹配：",
        "continuation revision 不能小于 0",
        "记忆候选 sidecar JSON 损坏，已拒绝覆盖 ",
        "记忆候选 store_version 不匹配：",
        "记忆候选 revision 不能小于 0",
    ];
    REJECTED_OWNER_ERROR_PREFIXES
        .iter()
        .any(|prefix| error.starts_with(prefix))
}

pub(crate) fn work_item_candidate(
    publication: &M4SourceOwnerOutboxEnvelopeV1,
    preliminary_scope_watermark: &str,
) -> Result<M4LegacyReadCandidate, String> {
    if publication.adapter_id != RegisteredWorkItemSourceOwnerMapper::ADAPTER_ID
        || publication.publication_kind != "WORK_ITEM_ATTENTION"
        || publication.object_type != M4_WORKFLOW_ATTENTION_OBJECT_TYPE
    {
        return Err("m4r06_work_item_publication_binding_invalid".to_string());
    }
    let mapped = RegisteredWorkItemSourceOwnerMapper::map(&publication.owner_status_code)?;
    if mapped.attention != publication.attention {
        return Err("m4r06_work_item_attention_mapping_drift".to_string());
    }
    let priority = m4_priority_reason(&M4AttentionSignals {
        external_commitment: publication.attention.external_commitment,
        time_sensitive: publication.attention.time_sensitive,
        requires_user_decision: publication.attention.requires_user_decision,
        source_blocked: publication.attention.source_blocked,
        attention_required: publication.attention.attention_required,
        material_change: publication.attention.material_change,
    })?;
    let legacy_item_ref = m4_internal_id(
        "legacy-item:sha256:",
        "syn.m4.r06.right-rail-work-item/v1",
        &[
            &publication.publication_id,
            &publication.canonical_object_id,
            &publication.source_event_id,
        ],
    )?;
    Ok(M4LegacyReadCandidate {
        legacy_source_kind: M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection,
        legacy_item_ref: Some(legacy_item_ref),
        source_owner_ref: Some(publication.source_owner_ref.clone()),
        scope_ref: Some(m4_primary_scope_ref().to_string()),
        source_type: Some(M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string()),
        canonical_source_object_id: Some(publication.canonical_object_id.clone()),
        source_revision: Some(publication.source_revision),
        source_owner_watermark: Some(publication.source_owner_watermark.clone()),
        source_link: Some(M4SourceLinkRead {
            link_kind: "INTERNAL_ROUTE".to_string(),
            source_owner_ref: publication.source_owner_ref.clone(),
            object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
            canonical_source_object_id: publication.canonical_object_id.clone(),
            expected_source_revision: publication.source_revision,
            opaque_route_ref: publication.opaque_route_ref.clone(),
        }),
        source_status_code: Some(mapped.source_status_code.to_string()),
        priority_reason_code: Some(priority.code.to_string()),
        scope_source_watermark: Some(preliminary_scope_watermark.to_string()),
    })
}

fn work_item_owner_cut_matches(
    initial_receipt: &M4LegacyReaderReceipt,
    initial_candidates: &[M4LegacyReadCandidate],
    final_receipt: &M4LegacyReaderReceipt,
    final_candidates: &[M4LegacyReadCandidate],
) -> bool {
    initial_receipt == final_receipt && initial_candidates == final_candidates
}

fn preliminary_work_item_owner_cut_watermark(
    initial_read: &M4LegacyServerOwnedReadBatch,
) -> Result<String, String> {
    let kind = M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection;
    let initial_candidates = initial_read
        .candidates
        .iter()
        .filter(|candidate| candidate.legacy_source_kind == kind)
        .collect::<Vec<_>>();
    let preliminary_scope_watermark = initial_candidates
        .first()
        .map(|candidate| {
            candidate
                .scope_source_watermark
                .as_deref()
                .ok_or_else(|| "m4r06_owner_cut_initial_watermark_missing".to_string())
        })
        .transpose()?
        .unwrap_or("");
    if initial_candidates.iter().any(|candidate| {
        candidate.scope_source_watermark.as_deref() != Some(preliminary_scope_watermark)
    }) {
        return Err("m4r06_owner_cut_initial_watermark_inconsistent".to_string());
    }
    Ok(preliminary_scope_watermark.to_string())
}

fn verify_work_item_owner_cut_against(
    initial_read: &M4LegacyServerOwnedReadBatch,
    final_receipt: &M4LegacyReaderReceipt,
    final_candidates: &[M4LegacyReadCandidate],
) -> Result<(), String> {
    let kind = M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection;
    let initial_receipt = initial_read
        .reader_receipts
        .iter()
        .find(|receipt| receipt.legacy_source_kind == kind.code())
        .ok_or_else(|| "m4r06_owner_cut_initial_receipt_missing".to_string())?;
    let initial_candidates = initial_read
        .candidates
        .iter()
        .filter(|candidate| candidate.legacy_source_kind == kind)
        .cloned()
        .collect::<Vec<_>>();
    if !work_item_owner_cut_matches(
        initial_receipt,
        &initial_candidates,
        final_receipt,
        final_candidates,
    ) {
        return Err("m4r06_work_item_owner_cut_changed".to_string());
    }
    Ok(())
}

#[cfg(test)]
trait ReadOnlyLegacyReaderFixture {
    fn read_surface(&self, kind: M4LegacyReadSourceKind) -> FixtureSurfaceRead;
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum FixtureSurfaceRead {
    Match { candidate_count: u64 },
    Empty,
    Unavailable { candidate_count: u64 },
    Rejected { candidate_count: u64 },
}

#[cfg(test)]
fn receipt_from_read_only_fixture(
    kind: M4LegacyReadSourceKind,
    fixture: &impl ReadOnlyLegacyReaderFixture,
) -> M4LegacyReaderReceipt {
    match fixture.read_surface(kind) {
        FixtureSurfaceRead::Match { candidate_count }
            if kind == M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection =>
        {
            M4LegacyReaderReceipt::new(
                kind,
                M4LegacyReaderReadState::Observed,
                None,
                Some(RegisteredWorkItemSourceOwnerMapper::ADAPTER_ID),
                candidate_count,
                candidate_count,
            )
        }
        FixtureSurfaceRead::Match { candidate_count } => unjoinable_receipt(kind, candidate_count),
        FixtureSurfaceRead::Empty => empty_receipt(kind),
        FixtureSurfaceRead::Unavailable { candidate_count } => M4LegacyReaderReceipt::new(
            kind,
            M4LegacyReaderReadState::Quarantined,
            Some(M4R06_READER_UNAVAILABLE),
            None,
            candidate_count,
            0,
        ),
        FixtureSurfaceRead::Rejected { candidate_count } => M4LegacyReaderReceipt::new(
            kind,
            M4LegacyReaderReadState::Quarantined,
            Some(M4R06_READER_REJECTED),
            None,
            candidate_count,
            0,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Fixture {
        surface: FixtureSurfaceRead,
    }

    impl ReadOnlyLegacyReaderFixture for Fixture {
        fn read_surface(&self, _kind: M4LegacyReadSourceKind) -> FixtureSurfaceRead {
            self.surface
        }
    }

    /// This fixture creates each source through its own local product store,
    /// then exercises only the reader.  It is intentionally separate from
    /// the receipt-only state matrix below: that matrix proves wire
    /// invariants, whereas these tests prove source-read behavior.
    struct RealLegacyReaderFixture {
        root: PathBuf,
        index_path: PathBuf,
        workflow_state_path: PathBuf,
        registry: Option<M4LegacyReadRegistry>,
    }

    impl RealLegacyReaderFixture {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos();
            let requested_root =
                std::env::temp_dir().join(format!("syn-m4c03-m4r06-{label}-{nanos}"));
            std::fs::create_dir_all(&requested_root).expect("create legacy reader fixture root");
            let root = std::fs::canonicalize(&requested_root)
                .expect("canonical legacy reader fixture root");
            let index_path = root.join("codex-index.json");
            let workflow_state_path = root.join("workflow-state.v0.json");
            std::fs::write(
                &index_path,
                serde_json::to_vec_pretty(&serde_json::json!({
                    "generated_at": "2026-08-11T00:00:00Z",
                    "projects": [],
                    "threads": []
                }))
                .expect("serialize empty legacy reader index"),
            )
            .expect("write empty legacy reader index");
            crate::initialize_workflow_state_at(&workflow_state_path)
                .expect("initialize valid empty workflow state");
            let repository = M4SecretarySqliteRepository::open_isolated_fixture(&root)
                .expect("open isolated M4 legacy reader repository");
            Self {
                root,
                index_path: index_path.clone(),
                workflow_state_path: workflow_state_path.clone(),
                registry: Some(M4LegacyReadRegistry::new(
                    &index_path,
                    &workflow_state_path,
                    repository,
                )),
            }
        }

        fn registry(&self) -> &M4LegacyReadRegistry {
            self.registry
                .as_ref()
                .expect("fixture registry remains installed")
        }

        fn bootstrap_project_workflow(&self) {
            let project_root = self.root.join("project");
            std::fs::create_dir_all(&project_root).expect("create fixture project root");
            let project = crate::ProjectRecord {
                project_root: project_root.display().to_string(),
                name: "M4R06 reader fixture".to_string(),
                active_hint: true,
                thread_count: 0,
                active_thread_count: 0,
                archived_thread_count: 0,
                latest_updated_at_ms: None,
                authority_files: vec![],
                handoff_files: vec![],
                evidence_files: vec![],
                harness_candidates: vec![],
                harness_resources: vec![],
                context_warnings: vec![],
                warnings: vec![],
            };
            crate::bootstrap_project_workflow_at(&self.workflow_state_path, &project)
                .expect("bootstrap fixture workflow through product owner entry");
        }

        fn create_memory_candidate(&self) {
            let project_root = self.root.join("memory-project");
            std::fs::create_dir_all(&project_root).expect("create fixture memory project root");
            let input = crate::CreateMemoryCandidateInput {
                project_root: project_root.display().to_string(),
                project_id: Some("project:m4r06-memory".to_string()),
                workflow_id: Some("workflow:m4r06-memory".to_string()),
                scope: crate::MemoryScope {
                    scope_id: "scope:project:m4r06-memory".to_string(),
                    scope_type: "project".to_string(),
                    user_id: None,
                    project_id: Some("project:m4r06-memory".to_string()),
                    workflow_id: Some("workflow:m4r06-memory".to_string()),
                    session_id: None,
                    role_ids: vec![],
                    document_refs: vec![],
                    permission_policy_ref: None,
                    model_export_policy: "local_only".to_string(),
                    valid_from: "2026-08-11T00:00:00Z".to_string(),
                    valid_until: None,
                },
                memory_type: "project_memory".to_string(),
                claim: "R06 memory reader fixture candidate".to_string(),
                body: "This candidate comes from the real memory candidate store writer."
                    .to_string(),
                source_refs: vec![crate::MemorySourceRef {
                    source_ref_id: "source:m4r06-memory".to_string(),
                    source_type: "reader_fixture".to_string(),
                    source_id: Some("fixture:m4r06-memory".to_string()),
                    source_path: Some("evidence/m4r06-memory.md".to_string()),
                    source_title: Some("M4R06 memory reader fixture".to_string()),
                    anchor: None,
                    source_created_at: None,
                    captured_at: "2026-08-11T00:00:00Z".to_string(),
                    authority_level: "evidence".to_string(),
                    sensitive_level: "project".to_string(),
                    content_hash: None,
                }],
                generated_by_role: "project_director".to_string(),
                generated_from: "m4r06_reader_test".to_string(),
                risk_level: "low".to_string(),
                sensitive_level: "project".to_string(),
                requires_user_confirmation: false,
                review_reason: "exercise read-only legacy reader classification".to_string(),
                expected_store_revision: None,
            };
            crate::memory_candidate_store::create_candidate(
                &self.workflow_state_path,
                &input,
                "2026-08-11T00:00:00Z",
                "m4r06-reader-fixture-write",
            )
            .expect("create fixture candidate through memory owner entry");
        }

        fn cleanup(mut self) {
            let root = self.root.clone();
            self.registry.take();
            drop(self);
            let _ = std::fs::remove_dir_all(root);
        }
    }

    fn receipt_for_kind<'a>(
        receipts: &'a [M4LegacyReaderReceipt],
        kind: M4LegacyReadSourceKind,
    ) -> &'a M4LegacyReaderReceipt {
        receipts
            .iter()
            .find(|receipt| receipt.legacy_source_kind == kind.code())
            .expect("fixed reader receipt exists")
    }

    fn assert_unavailable_source(receipt: &M4LegacyReaderReceipt) {
        assert_eq!(receipt.read_state, "QUARANTINED");
        assert_eq!(
            receipt.reason_code.as_deref(),
            Some(M4R06_READER_UNAVAILABLE)
        );
        assert_eq!(receipt.candidate_count, 0);
        assert_eq!(receipt.complete_tuple_count, 0);
    }

    fn assert_empty_source(receipt: &M4LegacyReaderReceipt) {
        assert_eq!(receipt.read_state, "EMPTY");
        assert_eq!(
            receipt.reason_code.as_deref(),
            Some(M4R06_EMPTY_SERVER_SURFACE)
        );
        assert_eq!(receipt.candidate_count, 0);
        assert_eq!(receipt.complete_tuple_count, 0);
    }

    fn assert_rejected_source(receipt: &M4LegacyReaderReceipt) {
        assert_eq!(receipt.read_state, "QUARANTINED");
        assert_eq!(receipt.reason_code.as_deref(), Some(M4R06_READER_REJECTED));
        assert_eq!(receipt.candidate_count, 0);
        assert_eq!(receipt.complete_tuple_count, 0);
    }

    fn existing_invalid_workflow_state_values(
        workflow_state_path: &Path,
    ) -> Vec<serde_json::Value> {
        let valid = crate::read_workflow_state_value(workflow_state_path)
            .expect("read initialized workflow-state fixture");

        let mut wrong_schema = valid.clone();
        wrong_schema["schema_version"] = serde_json::json!("workflow_state_v999");

        let mut missing_required_array = valid;
        missing_required_array
            .as_object_mut()
            .expect("workflow-state fixture is an object")
            .remove("harness_resources");

        vec![wrong_schema, missing_required_array]
    }

    fn write_workflow_state_fixture(path: &Path, value: &serde_json::Value) {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(value).expect("serialize workflow-state fixture"),
        )
        .expect("write workflow-state fixture");
    }

    fn write_memory_store_validator_fixture(
        workflow_state_path: &Path,
        store_version: &str,
        revision: i64,
    ) {
        let sidecar = crate::memory_candidate_store::sidecar_path(workflow_state_path)
            .expect("resolve memory validator fixture sidecar");
        std::fs::write(
            sidecar,
            serde_json::to_vec_pretty(&serde_json::json!({
                "store_version": store_version,
                "project_id": null,
                "workflow_id": null,
                "revision": revision,
                "candidates": [],
                "events": [],
                "updated_at": "2026-08-11T00:00:00Z"
            }))
            .expect("serialize memory validator fixture"),
        )
        .expect("write memory validator fixture");
    }

    fn write_continuation_store_validator_fixture(
        workflow_state_path: &Path,
        store_version: i64,
        storage_kind: &str,
        revision: i64,
    ) {
        let sidecar = crate::session_continuation_store::sidecar_path(workflow_state_path)
            .expect("resolve continuation validator fixture sidecar");
        std::fs::write(
            &sidecar,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": "session_continuation_store.v1",
                "store_version": store_version,
                "storage_kind": storage_kind,
                "scope": {
                    "scope_kind": "workflow_state_sidecar",
                    "workflow_state_path": workflow_state_path.display().to_string(),
                    "sidecar_path": sidecar.display().to_string(),
                    "project_roots": []
                },
                "revision": revision,
                "last_write_id": null,
                "generated_by": "m4r06-reader-fixture",
                "created_at": "2026-08-11T00:00:00Z",
                "updated_at": "2026-08-11T00:00:00Z",
                "continuations": [],
                "attempts": [],
                "audit_events": [],
                "warnings": []
            }))
            .expect("serialize continuation validator fixture"),
        )
        .expect("write continuation validator fixture");
    }

    #[test]
    fn m4r06_receipt_fixture_maps_every_kind_match_empty_unavailable_and_reject() {
        for kind in M4LegacyReadSourceKind::ALL {
            for surface in [
                FixtureSurfaceRead::Match { candidate_count: 3 },
                FixtureSurfaceRead::Empty,
                FixtureSurfaceRead::Unavailable { candidate_count: 2 },
                FixtureSurfaceRead::Rejected { candidate_count: 2 },
            ] {
                let receipt = receipt_from_read_only_fixture(kind, &Fixture { surface });
                assert_eq!(receipt.legacy_source_kind, kind.code());
                match surface {
                    FixtureSurfaceRead::Match { candidate_count }
                        if kind
                            == M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection =>
                    {
                        assert_eq!(receipt.read_state, "OBSERVED");
                        assert_eq!(receipt.reason_code, None);
                        assert_eq!(
                            receipt.legacy_reader_adapter_id.as_deref(),
                            Some(RegisteredWorkItemSourceOwnerMapper::ADAPTER_ID)
                        );
                        assert_eq!(receipt.candidate_count, candidate_count);
                        assert_eq!(receipt.complete_tuple_count, candidate_count);
                    }
                    FixtureSurfaceRead::Match { candidate_count } => {
                        assert_eq!(receipt.read_state, "UNJOINABLE");
                        assert_eq!(
                            receipt.reason_code.as_deref(),
                            Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE)
                        );
                        assert_eq!(receipt.candidate_count, candidate_count);
                        assert_eq!(receipt.complete_tuple_count, 0);
                    }
                    FixtureSurfaceRead::Empty => {
                        assert_eq!(receipt.read_state, "EMPTY");
                        assert_eq!(
                            receipt.reason_code.as_deref(),
                            Some(M4R06_EMPTY_SERVER_SURFACE)
                        );
                        assert_eq!(receipt.candidate_count, 0);
                        assert_eq!(receipt.complete_tuple_count, 0);
                    }
                    FixtureSurfaceRead::Unavailable { candidate_count } => {
                        assert_eq!(receipt.read_state, "QUARANTINED");
                        assert_eq!(
                            receipt.reason_code.as_deref(),
                            Some(M4R06_READER_UNAVAILABLE)
                        );
                        assert_eq!(receipt.candidate_count, candidate_count);
                        assert_eq!(receipt.complete_tuple_count, 0);
                    }
                    FixtureSurfaceRead::Rejected { candidate_count } => {
                        assert_eq!(receipt.read_state, "QUARANTINED");
                        assert_eq!(receipt.reason_code.as_deref(), Some(M4R06_READER_REJECTED));
                        assert_eq!(receipt.candidate_count, candidate_count);
                        assert_eq!(receipt.complete_tuple_count, 0);
                    }
                }
            }
        }
    }

    #[test]
    fn m4r06_secretary_reader_has_valid_empty_nonempty_unjoinable_and_invalid_rejected_paths() {
        let fixture = RealLegacyReaderFixture::new("secretary-reader-paths");
        let empty = fixture.registry().read_secretary_summary_primitives();
        assert_eq!(empty.read_state, "EMPTY");
        assert_eq!(
            empty.reason_code.as_deref(),
            Some(M4R06_EMPTY_SERVER_SURFACE)
        );

        fixture.bootstrap_project_workflow();
        let nonempty = fixture.registry().read_secretary_summary_primitives();
        assert_eq!(nonempty.read_state, "UNJOINABLE");
        assert_eq!(
            nonempty.reason_code.as_deref(),
            Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE)
        );
        assert!(nonempty.candidate_count > 0);
        assert_eq!(nonempty.complete_tuple_count, 0);

        std::fs::write(&fixture.workflow_state_path, "{ malformed workflow state")
            .expect("write malformed workflow source");
        let rejected = fixture.registry().read_secretary_summary_primitives();
        assert_eq!(rejected.read_state, "QUARANTINED");
        assert_eq!(rejected.reason_code.as_deref(), Some(M4R06_READER_REJECTED));
        assert_eq!(rejected.complete_tuple_count, 0);
        fixture.cleanup();
    }

    #[test]
    fn m4r06_secretary_reader_missing_is_empty_and_unreadable_quarantines_unavailable() {
        let fixture = RealLegacyReaderFixture::new("secretary-unavailable-source");
        std::fs::remove_file(&fixture.workflow_state_path)
            .expect("remove secretary source for missing-source read");
        assert_empty_source(&fixture.registry().read_secretary_summary_primitives());

        std::fs::create_dir(&fixture.workflow_state_path)
            .expect("replace secretary source with unreadable directory");
        assert_unavailable_source(&fixture.registry().read_secretary_summary_primitives());
        fixture.cleanup();
    }

    #[test]
    fn m4r06_secretary_reader_rejects_existing_wrong_schema_and_missing_required_array() {
        let fixture = RealLegacyReaderFixture::new("secretary-invalid-workflow-snapshot");
        for invalid_value in existing_invalid_workflow_state_values(&fixture.workflow_state_path) {
            write_workflow_state_fixture(&fixture.workflow_state_path, &invalid_value);
            let snapshot = crate::read_workflow_state_snapshot(&fixture.workflow_state_path)
                .expect("existing valid JSON still reads as a snapshot");
            assert!(snapshot.exists);
            assert!(!snapshot.initialized);
            assert_rejected_source(&fixture.registry().read_secretary_summary_primitives());
        }
        fixture.cleanup();
    }

    #[test]
    fn m4r06_runtime_reader_missing_continuation_is_empty_and_invalid_rejected_paths() {
        let fixture = RealLegacyReaderFixture::new("runtime-reader-paths");
        let continuation_sidecar =
            crate::session_continuation_store::sidecar_path(&fixture.workflow_state_path)
                .expect("resolve runtime continuation source");
        assert!(
            !continuation_sidecar.exists(),
            "fixture leaves the optional continuation source absent"
        );
        let empty = fixture
            .registry()
            .read_runtime_attention_primitives("1723334400000");
        assert_eq!(empty.read_state, "EMPTY");
        assert_eq!(
            empty.reason_code.as_deref(),
            Some(M4R06_EMPTY_SERVER_SURFACE)
        );

        std::fs::write(&fixture.index_path, "{ malformed index")
            .expect("write malformed runtime index");
        let rejected = fixture
            .registry()
            .read_runtime_attention_primitives("1723334400000");
        assert_eq!(rejected.read_state, "QUARANTINED");
        assert_eq!(rejected.reason_code.as_deref(), Some(M4R06_READER_REJECTED));
        assert_eq!(rejected.complete_tuple_count, 0);
        fixture.cleanup();
    }

    #[test]
    fn m4r06_runtime_reader_rejects_existing_wrong_schema_and_missing_required_array() {
        let fixture = RealLegacyReaderFixture::new("runtime-invalid-workflow-snapshot");
        for invalid_value in existing_invalid_workflow_state_values(&fixture.workflow_state_path) {
            write_workflow_state_fixture(&fixture.workflow_state_path, &invalid_value);
            let snapshot = crate::read_workflow_state_snapshot(&fixture.workflow_state_path)
                .expect("existing valid JSON still reads as a snapshot");
            assert!(snapshot.exists);
            assert!(!snapshot.initialized);
            assert_rejected_source(
                &fixture
                    .registry()
                    .read_runtime_attention_primitives("1723334400000"),
            );
        }
        fixture.cleanup();
    }

    #[test]
    fn m4r06_runtime_reader_missing_and_unreadable_index_quarantines_unavailable() {
        let fixture = RealLegacyReaderFixture::new("runtime-invalid-schema-io-missing");
        std::fs::remove_file(&fixture.index_path)
            .expect("remove runtime index for missing-source read");
        assert_unavailable_source(
            &fixture
                .registry()
                .read_runtime_attention_primitives("1723334400000"),
        );

        std::fs::create_dir(&fixture.index_path)
            .expect("replace runtime index with unreadable directory");
        assert_unavailable_source(
            &fixture
                .registry()
                .read_runtime_attention_primitives("1723334400000"),
        );
        fixture.cleanup();
    }

    #[test]
    fn m4r06_memory_reader_has_empty_candidate_unjoinable_and_invalid_rejected_paths() {
        let fixture = RealLegacyReaderFixture::new("memory-reader-paths");
        let empty = fixture
            .registry()
            .read_memory_daily_candidates("1723334400000");
        assert_eq!(empty.read_state, "EMPTY");
        assert_eq!(
            empty.reason_code.as_deref(),
            Some(M4R06_EMPTY_SERVER_SURFACE)
        );

        fixture.create_memory_candidate();
        let candidate = fixture
            .registry()
            .read_memory_daily_candidates("1723334400000");
        assert_eq!(candidate.read_state, "UNJOINABLE");
        assert_eq!(
            candidate.reason_code.as_deref(),
            Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE)
        );
        assert!(candidate.candidate_count > 0);
        assert_eq!(candidate.complete_tuple_count, 0);

        let sidecar = crate::memory_candidate_store::sidecar_path(&fixture.workflow_state_path)
            .expect("resolve memory source sidecar");
        std::fs::write(sidecar, "{ malformed memory candidate store")
            .expect("write malformed memory source");
        let rejected = fixture
            .registry()
            .read_memory_daily_candidates("1723334400000");
        assert_eq!(rejected.read_state, "QUARANTINED");
        assert_eq!(rejected.reason_code.as_deref(), Some(M4R06_READER_REJECTED));
        assert_eq!(rejected.complete_tuple_count, 0);
        fixture.cleanup();
    }

    #[test]
    fn m4r06_memory_reader_rejects_store_version_and_revision_validator_errors() {
        let fixture = RealLegacyReaderFixture::new("memory-validator-errors");
        for (store_version, revision) in [
            ("memory_candidate_store.v0", 0),
            ("memory_candidate_store.v1", -1),
        ] {
            write_memory_store_validator_fixture(
                &fixture.workflow_state_path,
                store_version,
                revision,
            );
            assert_rejected_source(
                &fixture
                    .registry()
                    .read_memory_daily_candidates("1723334400000"),
            );
        }
        fixture.cleanup();
    }

    #[test]
    fn m4r06_memory_reader_missing_is_empty_and_unreadable_quarantines_unavailable() {
        let fixture = RealLegacyReaderFixture::new("memory-unavailable-source");
        let sidecar = crate::memory_candidate_store::sidecar_path(&fixture.workflow_state_path)
            .expect("resolve memory source sidecar");
        assert!(
            !sidecar.exists(),
            "fixture leaves the optional memory source absent"
        );
        assert_empty_source(
            &fixture
                .registry()
                .read_memory_daily_candidates("1723334400000"),
        );

        std::fs::create_dir(sidecar).expect("replace memory source with unreadable directory");
        assert_unavailable_source(
            &fixture
                .registry()
                .read_memory_daily_candidates("1723334400000"),
        );
        fixture.cleanup();
    }

    #[test]
    fn m4r06_runtime_reader_rejects_continuation_storage_kind_store_version_and_revision_errors() {
        let fixture = RealLegacyReaderFixture::new("runtime-continuation-validator-errors");
        for (store_version, storage_kind, revision) in [
            (1, "sidecar_json_v0:invalid", 0),
            (2, "sidecar_json_v0", 0),
            (1, "sidecar_json_v0", -1),
        ] {
            write_continuation_store_validator_fixture(
                &fixture.workflow_state_path,
                store_version,
                storage_kind,
                revision,
            );
            assert_rejected_source(
                &fixture
                    .registry()
                    .read_runtime_attention_primitives("1723334400000"),
            );
        }
        fixture.cleanup();
    }

    #[test]
    fn m4r06_registry_keeps_react_fixed_unjoinable_and_owner_unavailable_quarantined() {
        let fixture = RealLegacyReaderFixture::new("fixed-react-owner-unavailable");
        let batch = fixture
            .registry()
            .read_server_owned_legacy_candidates()
            .expect("M4 preliminary watermark and fixed readers are readable");
        let right_rail = receipt_for_kind(
            &batch.reader_receipts,
            M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection,
        );
        assert_eq!(right_rail.read_state, "QUARANTINED");
        assert_eq!(
            right_rail.reason_code.as_deref(),
            Some(M4R06_READER_UNAVAILABLE)
        );
        assert_eq!(right_rail.complete_tuple_count, 0);

        // React has no server-owned tuple source.  The registry fixes this
        // receipt without reading any renderer/localStorage input.
        let react = receipt_for_kind(
            &batch.reader_receipts,
            M4LegacyReadSourceKind::ReactPendingActionVisibility,
        );
        assert_eq!(react.read_state, "UNJOINABLE");
        assert_eq!(
            react.reason_code.as_deref(),
            Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE)
        );
        assert_eq!(react.candidate_count, 0);
        assert_eq!(react.complete_tuple_count, 0);
        assert!(batch.candidates.is_empty());
        fixture.cleanup();
    }

    #[test]
    fn m4r06_ready_to_dispatch_uses_registered_mapper_attention_semantics() {
        assert!(
            crate::workbench_sqlite_repository::m4r06_work_item_status_is_attention_eligible(
                "ready_to_dispatch",
            )
            .expect("registered ready_to_dispatch mapper")
        );
        assert!(
            !crate::workbench_sqlite_repository::m4r06_work_item_status_is_attention_eligible(
                "accepted",
            )
            .expect("registered terminal mapper")
        );
    }

    fn m4r06_owner_cut_candidate(
        source_revision: u64,
        source_owner_watermark: &str,
    ) -> M4LegacyReadCandidate {
        M4LegacyReadCandidate {
            legacy_source_kind: M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection,
            legacy_item_ref: Some(format!("legacy-item:{source_revision}")),
            source_owner_ref: Some(
                crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_OWNER_REF.to_string(),
            ),
            scope_ref: Some(m4_primary_scope_ref().to_string()),
            source_type: Some(M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string()),
            canonical_source_object_id: Some("work-item:m4r06-owner-cut".to_string()),
            source_revision: Some(source_revision),
            source_owner_watermark: Some(source_owner_watermark.to_string()),
            source_link: Some(M4SourceLinkRead {
                link_kind: "INTERNAL_ROUTE".to_string(),
                source_owner_ref: crate::m4_source_owner_schema::M4_WORK_ITEM_SOURCE_OWNER_REF
                    .to_string(),
                object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
                canonical_source_object_id: "work-item:m4r06-owner-cut".to_string(),
                expected_source_revision: source_revision,
                opaque_route_ref: "route:m4r06-owner-cut".to_string(),
            }),
            source_status_code: Some("OPEN".to_string()),
            priority_reason_code: Some("attention_required".to_string()),
            scope_source_watermark: Some("m4-watermark:before-owner-advance".to_string()),
        }
    }

    #[test]
    fn m4r06_owner_advance_after_first_read_without_m4_delivery_blocks_parity_cut() {
        let kind = M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection;
        let initial_receipt = M4LegacyReaderReceipt::new(
            kind,
            M4LegacyReaderReadState::Observed,
            None,
            Some(RegisteredWorkItemSourceOwnerMapper::ADAPTER_ID),
            1,
            1,
        );
        let initial_read = M4LegacyServerOwnedReadBatch {
            reader_receipts: vec![initial_receipt.clone()],
            candidates: vec![m4r06_owner_cut_candidate(7, "owner-watermark:7")],
        };

        // The owner has advanced after the first read, while M4 intentionally
        // remains at the pre-advance canonical watermark.  The receipt is
        // still OBSERVED, so comparing only receipt state/counts would be a
        // fail-open; the complete tuple must force the envelope unavailable.
        let final_candidates = vec![m4r06_owner_cut_candidate(8, "owner-watermark:8")];
        assert_eq!(
            verify_work_item_owner_cut_against(&initial_read, &initial_receipt, &final_candidates,)
                .expect_err("owner advance must invalidate the WorkItem cut"),
            "m4r06_work_item_owner_cut_changed"
        );
    }

    #[test]
    fn m4r06_preview_only_runtime_attention_is_unjoinable_not_empty() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let requested_root =
            std::env::temp_dir().join(format!("syn-m4c03-m4r06-runtime-preview-{nanos}"));
        std::fs::create_dir_all(&requested_root).expect("create runtime reader fixture root");
        let root =
            std::fs::canonicalize(&requested_root).expect("canonical runtime reader fixture root");
        let index_path = root.join("codex-index.json");
        let workflow_state_path = root.join("workflow-state.v0.json");
        let project_root = root.join("project-root");
        std::fs::create_dir_all(&project_root).expect("create fixture project root");
        std::fs::write(
            &index_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "generated_at": "2026-08-11T00:00:00Z",
                "projects": [{
                    "project_root": project_root.display().to_string(),
                    "harness_resources": [{
                        "root_path": project_root.join("harness").display().to_string(),
                        "adapter_id": "codex-local"
                    }]
                }],
                "threads": [{
                    "thread_id": "thread:m4r06-preview",
                    "title": "R06 preview-only session",
                    "project_root": project_root.display().to_string(),
                    "thread_source": "codex",
                    "rollout_path": project_root.join("preview.jsonl").display().to_string(),
                    "rollout_exists": true
                }]
            }))
            .expect("serialize runtime reader index"),
        )
        .expect("write runtime reader index");
        std::fs::write(
            &workflow_state_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": "workflow_state_v0",
                "workflow_version": 1,
                "workspace_id": "workspace:m4r06-preview",
                "updated_at": "2026-08-11T00:00:00Z",
                "projects": [{
                    "project_id": "project:m4r06-preview",
                    "root_path": project_root.display().to_string()
                }],
                "agent_adapters": [{"adapter_id": "codex-local", "agent_type": "codex"}],
                "workflows": [{
                    "workflow_id": "workflow:m4r06-preview",
                    "project_id": "project:m4r06-preview",
                    "title": "R06 preview",
                    "state": "running"
                }],
                "nodes": [{
                    "node_id": "node:m4r06-preview",
                    "workflow_id": "workflow:m4r06-preview",
                    "node_type": "dev_line",
                    "title": "R06 preview node",
                    "state": "running"
                }],
                "edges": [],
                "work_items": [{
                    "work_item_id": "work:m4r06-preview",
                    "workflow_id": "workflow:m4r06-preview",
                    "project_id": "project:m4r06-preview",
                    "title": "Preview-only runtime attention",
                    "state": "ready_to_dispatch"
                }],
                "artifacts": [],
                "reviews": [],
                "audit_events": [],
                "capabilities": [],
                "harness_resources": [],
                "workflow_node_session_bindings": [{
                    "binding_id": "binding:m4r06-preview",
                    "project_id": "project:m4r06-preview",
                    "workflow_id": "workflow:m4r06-preview",
                    "node_id": "node:m4r06-preview",
                    "work_item_id": "work:m4r06-preview",
                    "agent_type": "codex",
                    "adapter_id": "codex-local",
                    "native_thread_id": "thread:m4r06-preview",
                    "native_rollout_path": project_root.join("preview.jsonl").display().to_string(),
                    "session_title": "R06 preview-only session",
                    "rollout_exists": true,
                    "lifecycle": "active",
                    "created_at_ms": 1,
                    "updated_at_ms": 2
                }],
                "workflow_node_dispatches": [],
                "workflow_execution_controls": [],
                "permission_requests": [],
                "execution_attempts": []
            }))
            .expect("serialize runtime reader workflow state"),
        )
        .expect("write runtime reader workflow state");

        let repository = M4SecretarySqliteRepository::open_isolated_fixture(&root)
            .expect("open isolated M4 fixture for runtime reader");
        let registry = M4LegacyReadRegistry::new(&index_path, &workflow_state_path, repository);
        let receipt = registry.read_runtime_attention_primitives("1723334400000");
        assert_eq!(receipt.read_state, "UNJOINABLE");
        assert_eq!(
            receipt.reason_code.as_deref(),
            Some(M4R06_UNJOINABLE_NO_EXACT_TUPLE)
        );
        assert!(
            receipt.candidate_count > 0,
            "preview attention was observed"
        );
        assert_eq!(receipt.complete_tuple_count, 0);

        let _ = std::fs::remove_dir_all(root);
    }
}
