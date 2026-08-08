use crate::utils::hash::{sha256_hex, sha256_hex_bytes};
use crate::workbench_sqlite_repository::{
    ConfirmedWorkbenchSqliteRepositoryConfig, RepositoryAuditEntry, WorkbenchSqliteRepository,
    WorkflowStateSidecarQuarantineManifestEntry, WorkflowStateSidecarQuarantineReason,
    CONFIRMED_DB_DENIED_PATH_MARKERS,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[path = "workbench_sqlite_storage_mode_m5c.rs"]
mod m5c;

#[path = "workbench_sqlite_storage_mode_m5f1.rs"]
mod m5f1;

// A·只读访问器（系统状态读模型用）。单独成文件而非挂在本文件里：本文件已贴着 shape gate 的
// 3000 行上限（加 16 行就破线·gate 当场抓到），故照 m5c 先例拆子模块。
#[path = "workbench_sqlite_storage_mode_read_model.rs"]
mod read_model;

pub(crate) use m5f1::{
    primary_repository_for_m2_t2_fail_closed_write, primary_repository_for_write,
    workflow_state_write_route, WorkflowStateWriteRoute,
};
pub(crate) use read_model::db_primary_health_snapshot;

pub(crate) const STORAGE_MODE_SCHEMA_VERSION: &str = "storage-mode.v1";
pub(crate) const STORAGE_MODE_FILE_NAME: &str = "storage-mode.v1.json";
const JSON_ONLY: &str = "json_only";
const DB_PRIMARY_JSON_PROJECTION: &str = "db_primary_json_projection";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StorageMode {
    JsonOnly { reason: String },
    DbPrimaryJsonProjection(DbPrimaryJsonProjectionConfig),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DbPrimaryJsonProjectionConfig {
    pub(crate) workflow_state_path: PathBuf,
    pub(crate) confirmed_workflow_state_path: PathBuf,
    pub(crate) db_path: PathBuf,
    pub(crate) confirmed_db_path: PathBuf,
    pub(crate) denied_path_markers: Vec<String>,
}

impl DbPrimaryJsonProjectionConfig {
    pub(crate) fn db_path_hash(&self) -> String {
        sha256_hex(&self.db_path.display().to_string())
    }

    fn repository_config(&self) -> ConfirmedWorkbenchSqliteRepositoryConfig {
        ConfirmedWorkbenchSqliteRepositoryConfig {
            db_path: self.db_path.clone(),
            confirmed_db_path: self.confirmed_db_path.clone(),
            denied_path_markers: self.denied_path_markers.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StorageModeFileV1 {
    schema_version: String,
    mode: String,
    #[serde(default)]
    workflow_state_path: Option<PathBuf>,
    #[serde(default)]
    confirmed_workflow_state_path: Option<PathBuf>,
    #[serde(default)]
    db_path: Option<PathBuf>,
    #[serde(default)]
    confirmed_db_path: Option<PathBuf>,
    #[serde(default)]
    denied_path_markers: Vec<String>,
}

#[derive(Clone, Debug)]
enum DbPrimaryHealth {
    Ready,
    Blocked(String),
}

fn mode_cache() -> &'static Mutex<BTreeMap<PathBuf, StorageMode>> {
    static CACHE: OnceLock<Mutex<BTreeMap<PathBuf, StorageMode>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn health_cache() -> &'static Mutex<BTreeMap<PathBuf, DbPrimaryHealth>> {
    static CACHE: OnceLock<Mutex<BTreeMap<PathBuf, DbPrimaryHealth>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

// A blocked DB primary writer falls back to the established JSON path. Record that transition
// once for the process so repeated product writes do not flood the workflow audit or stderr.
fn degradation_audit_recorded() -> &'static Mutex<bool> {
    static RECORDED: OnceLock<Mutex<bool>> = OnceLock::new();
    RECORDED.get_or_init(|| Mutex::new(false))
}

pub(crate) fn storage_mode_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    let state_parent = workflow_state_path.parent().ok_or_else(|| {
        format!(
            "storage_mode_workflow_state_parent_required:{}",
            workflow_state_path.display()
        )
    })?;
    let app_root =
        if state_parent.file_name().and_then(|name| name.to_str()) == Some("workflow-state") {
            state_parent.parent().unwrap_or(state_parent)
        } else {
            state_parent
        };
    Ok(app_root
        .join("runtime-artifacts")
        .join(STORAGE_MODE_FILE_NAME))
}

// Mode is intentionally cached by workflow-state path. A config change takes effect only after
// process restart, so a running app cannot hot-switch its primary writer mid-operation.
pub(crate) fn storage_mode_for(workflow_state_path: &Path) -> StorageMode {
    let key = workflow_state_path.to_path_buf();
    let mut cache = mode_cache().lock().expect("storage mode cache lock");
    if let Some(mode) = cache.get(&key) {
        return mode.clone();
    }
    let mode = resolve_storage_mode(workflow_state_path);
    cache.insert(key, mode.clone());
    mode
}

// Once a DB commit has succeeded, a failed JSON projection must stop this process from
// extending the DB-leading window. Startup reconciliation is the only recovery path.
pub(crate) fn complete_db_primary_json_projection<T>(
    workflow_state_path: &Path,
    phase: &str,
    projection: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    match projection() {
        Ok(value) => Ok(value),
        Err(error) => {
            block_db_primary_writes(workflow_state_path, phase, error.clone());
            Err(error)
        }
    }
}

pub(crate) fn block_db_primary_writes(
    workflow_state_path: &Path,
    phase: &str,
    reason: impl Into<String>,
) {
    health_cache()
        .lock()
        .expect("storage mode health lock")
        .insert(
            workflow_state_path.to_path_buf(),
            DbPrimaryHealth::Blocked(format!(
                "db_primary_json_projection_failed:{phase}:{}",
                reason.into()
            )),
        );
}

pub(crate) fn initialize_for_startup(workflow_state_path: &Path) -> Result<(), String> {
    let mode = storage_mode_for(workflow_state_path);
    match mode {
        StorageMode::JsonOnly { reason } => {
            eprintln!("storage mode=json_only reason={reason}");
            Ok(())
        }
        StorageMode::DbPrimaryJsonProjection(config) => {
            eprintln!(
                "storage mode=db_primary_json_projection db_path_hash={}",
                config.db_path_hash()
            );
            let startup = (|| {
                quarantine_m2_workflow_state_sidecar_if_needed(&config)?;
                require_no_unresolved_m2_workflow_state_sidecar_quarantine(&config)?;
                let report = reconcile_db_vs_json(&config)?;
                if report.has_json_leading_or_divergence() {
                    return Err(report.fail_closed_reason());
                }
                let replayed_db_primary_projection = report.has_db_leading();
                if replayed_db_primary_projection {
                    // The workflow-state JSON file is an internal,
                    // rebuildable projection.  It deliberately has no
                    // external-effect lease or result-command state: startup
                    // replays the authoritative DB snapshot directly and
                    // verifies parity below.
                    replay_db_primary_projection(&config)?;
                    let replayed = reconcile_db_vs_json(&config)?;
                    if !replayed.is_green() {
                        return Err(replayed.fail_closed_reason());
                    }
                }
                append_startup_mode_audit(&config, replayed_db_primary_projection)?;
                let final_report = reconcile_db_vs_json(&config)?;
                if !final_report.is_green() {
                    return Err(final_report.fail_closed_reason());
                }
                repair_m2_workflow_state_sidecar_checkpoint_after_startup(&config)?;
                Ok(())
            })();
            let mut health = health_cache().lock().expect("storage mode health lock");
            match startup {
                Ok(()) => {
                    health.insert(workflow_state_path.to_path_buf(), DbPrimaryHealth::Ready);
                    Ok(())
                }
                Err(error) => {
                    health.insert(
                        workflow_state_path.to_path_buf(),
                        DbPrimaryHealth::Blocked(error.clone()),
                    );
                    eprintln!(
                        "storage mode=db_primary_json_projection blocked; 已降级 json_only，数据无损，需重 seed 恢复 DB 主写；reason={error}"
                    );
                    Err(format!("db_primary_projection_blocked:{error}"))
                }
            }
        }
    }
}

/// The M2 reference slice has exactly one legacy owner: the workflow-state
/// JSON sidecar.  This adapter examines only its structural envelope and
/// records a reference-only quarantine before any input which is corrupt,
/// sensitive, unknown, or unjoinable can reach ordinary SQLite tables.
///
/// It intentionally does not inspect values for output, import a raw payload,
/// or provide a caller-selected migration path.  Repair/rebuild remains an
/// explicit future operation over the retained original source.
fn quarantine_m2_workflow_state_sidecar_if_needed(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<(), String> {
    let Some(observation) = inspect_m2_workflow_state_sidecar(config)? else {
        return Ok(());
    };
    let repository = WorkbenchSqliteRepository::open_confirmed(&config.repository_config())?;
    let (entry, _) = repository
        .with_immediate_transaction(
            "m2_workflow_state_sidecar_quarantine",
            None,
            |transaction| {
                crate::workbench_sqlite_repository::quarantine_m2_workflow_state_sidecar_in_transaction(
                    transaction,
                    &observation.source_sha256,
                    observation.reason,
                    crate::unix_timestamp_ms(),
                )
            },
        )
        .map_err(|error| format!("m2_workflow_state_sidecar_quarantine:{error}"))?;
    let current_sha256 = sha256_hex_bytes(
        &fs::read(&config.workflow_state_path)
            .map_err(|error| format!("m2_workflow_state_sidecar_quarantine_recheck:{error}"))?,
    );
    if current_sha256 != observation.source_sha256 {
        return Err("m2_workflow_state_sidecar_quarantine_input_changed_during_record".to_string());
    }
    let expected_source_ref = format!("workflow-state-sidecar:sha256:{current_sha256}");
    if entry.source_ref != expected_source_ref
        || entry.scope_ref
            != crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE
        || entry.reason_code != observation.reason.code()
    {
        return Err("m2_workflow_state_sidecar_quarantine_receipt_mismatch".to_string());
    }
    Err(format!(
        "m2_workflow_state_sidecar_quarantined:{}:{}",
        entry.reason_code, entry.quarantine_id
    ))
}

/// Value-free only: callers can enumerate retained M2 reference-slice
/// quarantine metadata for an authorized repair or export decision, but never
/// source bytes, paths, field names, or original values.
pub(crate) fn m2_workflow_state_sidecar_quarantine_manifest(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<Vec<WorkflowStateSidecarQuarantineManifestEntry>, String> {
    WorkbenchSqliteRepository::open_confirmed(&config.repository_config())?
        .m2_workflow_state_sidecar_quarantine_manifest()
}

/// This is not an automatic startup repair and is not exposed as a product
/// command.  A controlled caller must name the already-exported quarantine
/// receipt.  The current sidecar must independently pass the M2 envelope and
/// DB/JSON reconciliation before its retained record can become `REBUILT`.
pub(crate) fn rebuild_m2_workflow_state_sidecar_quarantine(
    config: &DbPrimaryJsonProjectionConfig,
    quarantine_id: &str,
) -> Result<WorkflowStateSidecarQuarantineManifestEntry, String> {
    if inspect_m2_workflow_state_sidecar(config)?.is_some() {
        return Err("m2_workflow_state_sidecar_quarantine_rebuild_current_input_not_green".to_string());
    }
    let report = reconcile_db_vs_json(config)?;
    if !report.is_green() {
        return Err("m2_workflow_state_sidecar_quarantine_rebuild_reconciliation_not_green".to_string());
    }
    let current_sha256 = sha256_hex_bytes(
        &fs::read(&config.workflow_state_path)
            .map_err(|error| format!("m2_workflow_state_sidecar_quarantine_rebuild_read:{error}"))?,
    );
    let repository = WorkbenchSqliteRepository::open_confirmed(&config.repository_config())?;
    let (entry, _) = repository
        .with_immediate_transaction(
            "m2_workflow_state_sidecar_quarantine_rebuild",
            None,
            |transaction| {
                crate::workbench_sqlite_repository::rebuild_m2_workflow_state_sidecar_quarantine_in_transaction(
                    transaction,
                    quarantine_id,
                    &current_sha256,
                    crate::unix_timestamp_ms(),
                )
            },
        )
        .map_err(|error| format!("m2_workflow_state_sidecar_quarantine_rebuild:{error}"))?;
    let rechecked_sha256 = sha256_hex_bytes(
        &fs::read(&config.workflow_state_path).map_err(|error| {
            format!("m2_workflow_state_sidecar_quarantine_rebuild_recheck:{error}")
        })?,
    );
    if rechecked_sha256 != current_sha256 || entry.resolution_state != "REBUILT" {
        return Err("m2_workflow_state_sidecar_quarantine_rebuild_receipt_mismatch".to_string());
    }
    Ok(entry)
}

fn require_no_unresolved_m2_workflow_state_sidecar_quarantine(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<(), String> {
    let unresolved = m2_workflow_state_sidecar_quarantine_manifest(config)?
        .into_iter()
        .filter(|entry| entry.resolution_state == "PENDING" || entry.resolution_state == "HELD")
        .map(|entry| entry.quarantine_id)
        .collect::<Vec<_>>();
    if unresolved.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "m2_workflow_state_sidecar_unresolved_quarantine:{}",
            unresolved.join(",")
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct M2WorkflowStateSidecarObservation {
    source_sha256: String,
    reason: WorkflowStateSidecarQuarantineReason,
}

fn inspect_m2_workflow_state_sidecar(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<Option<M2WorkflowStateSidecarObservation>, String> {
    let bytes = fs::read(&config.workflow_state_path)
        .map_err(|error| format!("m2_workflow_state_sidecar_read:{error}"))?;
    let source_sha256 = sha256_hex_bytes(&bytes);
    let observation = |reason| {
        Some(M2WorkflowStateSidecarObservation {
            source_sha256: source_sha256.clone(),
            reason,
        })
    };
    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return Ok(observation(WorkflowStateSidecarQuarantineReason::CorruptInput)),
    };
    let Some(object) = value.as_object() else {
        return Ok(observation(
            WorkflowStateSidecarQuarantineReason::UnjoinableReferenceRecord,
        ));
    };
    if !crate::validate_workflow_state(&value).is_empty() {
        return Ok(observation(
            WorkflowStateSidecarQuarantineReason::UnjoinableReferenceRecord,
        ));
    }
    for field in object.keys() {
        if m2_workflow_state_sidecar_sensitive_root_field(field) {
            return Ok(observation(WorkflowStateSidecarQuarantineReason::SensitiveInput));
        }
        if !M2_WORKFLOW_STATE_SIDECAR_ALLOWED_ROOT_FIELDS.contains(&field.as_str()) {
            return Ok(observation(WorkflowStateSidecarQuarantineReason::UnknownInput));
        }
    }
    for (array_name, key_field) in [
        ("work_items", "work_item_id"),
        ("nodes", "node_id"),
        ("workflow_node_session_bindings", "binding_id"),
        ("workflow_node_dispatches", "dispatch_id"),
    ] {
        if !m2_workflow_state_sidecar_records_joinable(&value, array_name, key_field) {
            return Ok(observation(
                WorkflowStateSidecarQuarantineReason::UnjoinableReferenceRecord,
            ));
        }
    }
    Ok(None)
}

const M2_WORKFLOW_STATE_SIDECAR_ALLOWED_ROOT_FIELDS: &[&str] = &[
    "schema_version",
    "workflow_version",
    "revision",
    "workspace_id",
    "created_at",
    "updated_at",
    "source_kind",
    "permission_level",
    "projects",
    "agent_adapters",
    "workflows",
    "nodes",
    "edges",
    "work_items",
    "artifacts",
    "reviews",
    "workflow_node_session_bindings",
    "workflow_node_dispatches",
    "audit_events",
    "capabilities",
    "harness_resources",
    // These are forward compatibility containers.  The M2 adapter does not
    // interpret them, but recognizes their fixed public envelope so a later
    // stage cannot be mistaken for a raw unknown input.
    "execution_attempts",
    "workflow_chain_runs",
    "workflow_execution_controls",
    "permission_requests",
];

fn m2_workflow_state_sidecar_sensitive_root_field(field: &str) -> bool {
    matches!(
        field.to_ascii_lowercase().as_str(),
        "secret"
            | "secrets"
            | "secret_value"
            | "token"
            | "tokens"
            | "credential"
            | "credentials"
            | "credential_token"
            | "oauth"
            | "api_key"
            | "private_key"
            | "prompt_body"
            | "full_transcript"
    )
}

fn m2_workflow_state_sidecar_records_joinable(
    value: &Value,
    array_name: &str,
    key_field: &str,
) -> bool {
    let Some(array) = value.get(array_name) else {
        // These two arrays are optional in the frozen v0 sidecar shape.  If
        // present, however, every record must have a unique exact join key.
        return true;
    };
    let Some(array) = array.as_array() else {
        return false;
    };
    let mut keys = BTreeSet::new();
    array.iter().all(|record| {
        record
            .get(key_field)
            .and_then(Value::as_str)
            .filter(|key| !key.is_empty())
            .is_some_and(|key| keys.insert(key.to_string()))
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DbJsonTableReconciliation {
    pub(crate) table_name: String,
    pub(crate) db_count: usize,
    pub(crate) json_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) db_leading: Vec<String>,
    pub(crate) json_leading: Vec<String>,
    pub(crate) hash_mismatches: Vec<String>,
    pub(crate) db_unprojected_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DbJsonReconciliationReport {
    pub(crate) status: String,
    pub(crate) tables: Vec<DbJsonTableReconciliation>,
}

impl DbJsonReconciliationReport {
    pub(crate) fn is_green(&self) -> bool {
        self.tables.iter().all(|table| {
            table.db_leading.is_empty()
                && table.json_leading.is_empty()
                && table.hash_mismatches.is_empty()
        })
    }

    fn has_db_leading(&self) -> bool {
        self.tables.iter().any(|table| !table.db_leading.is_empty())
    }

    fn has_json_leading_or_divergence(&self) -> bool {
        self.tables
            .iter()
            .any(|table| !table.json_leading.is_empty() || !table.hash_mismatches.is_empty())
    }

    fn fail_closed_reason(&self) -> String {
        let details = self
            .tables
            .iter()
            .filter(|table| {
                !table.db_leading.is_empty()
                    || !table.json_leading.is_empty()
                    || !table.hash_mismatches.is_empty()
            })
            .map(|table| {
                format!(
                    "{}:db_leading={:?}:json_leading={:?}:hash_mismatches={:?}",
                    table.table_name, table.db_leading, table.json_leading, table.hash_mismatches
                )
            })
            .collect::<Vec<_>>();
        format!("db_json_reconciliation_not_green:{}", details.join("|"))
    }
}

#[derive(Clone, Debug)]
struct DbRecord {
    natural_key: String,
    record_hash: String,
    value: Value,
}

#[derive(Clone, Debug)]
struct DbAuditRecord {
    record: DbRecord,
    target_kind: String,
}

#[derive(Clone, Debug, Default)]
struct DbProjectionData {
    proposals: Vec<DbRecord>,
    proposal_audits: Vec<DbRecord>,
    authorizations: Vec<DbRecord>,
    authorization_audits: Vec<DbRecord>,
    projects: Vec<DbRecord>,
    agent_adapters: Vec<DbRecord>,
    workflows: Vec<DbRecord>,
    nodes: Vec<DbRecord>,
    edges: Vec<DbRecord>,
    work_items: Vec<DbRecord>,
    artifacts: Vec<DbRecord>,
    reviews: Vec<DbRecord>,
    bindings: Vec<DbRecord>,
    dispatches: Vec<DbRecord>,
    execution_attempts: Vec<DbRecord>,
    chain_runs: Vec<DbRecord>,
    execution_controls: Vec<DbRecord>,
    permission_requests: Vec<DbRecord>,
    capabilities: Vec<DbRecord>,
    harness_resources: Vec<DbRecord>,
    supervisor_actions: Vec<DbRecord>,
    supervisor_orchestrator_sessions: Vec<DbRecord>,
    supervisor_orchestrator_audit_events: Vec<DbRecord>,
    m5c: m5c::M5cDbProjectionData,
    audits: Vec<DbAuditRecord>,
}

pub(crate) fn reconcile_db_vs_json(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<DbJsonReconciliationReport, String> {
    let database = load_db_projection_data(config)?;
    let proposal_store = crate::project_consultation_proposal_store::load_store(
        &config.workflow_state_path,
        crate::unix_timestamp_ms(),
    )?;
    let authorization_store = crate::plan_authorization_store::load_store(
        &config.workflow_state_path,
        crate::unix_timestamp_ms(),
    )?;
    let workflow_state = crate::read_workflow_state_value(&config.workflow_state_path)?;
    let supervisor_actions = crate::supervisor_action_controller::db_primary_projection_records(
        &config.workflow_state_path,
    )?;
    let (supervisor_orchestrator_sessions, supervisor_orchestrator_audit_events) =
        crate::mcp::supervisor_orchestrator::db_primary_projection_records(
            &config.workflow_state_path,
        )?;

    let proposal_records = values_to_records(
        proposal_store
            .proposals
            .into_iter()
            .map(|value| serde_json::to_value(value).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        "proposal_id",
    )?;
    let authorization_records = values_to_records(
        authorization_store
            .authorizations
            .into_iter()
            .map(|value| serde_json::to_value(value).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        "authorization_id",
    )?;
    let proposal_audits = values_to_records(
        proposal_store
            .audit_events
            .into_iter()
            .map(|value| serde_json::to_value(value).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        "audit_event_id",
    )?;
    let authorization_audits = values_to_records(
        authorization_store
            .audit_events
            .into_iter()
            .map(|value| serde_json::to_value(value).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?,
        "audit_event_id",
    )?;
    let workflow_projects = array_records(&workflow_state, "projects", "project_id")?;
    let workflow_agent_adapters = array_records(&workflow_state, "agent_adapters", "adapter_id")?;
    let workflow_workflows = array_records(&workflow_state, "workflows", "workflow_id")?;
    let workflow_nodes = array_records(&workflow_state, "nodes", "node_id")?;
    let workflow_edges = array_records(&workflow_state, "edges", "edge_id")?;
    let workflow_work_items = array_records(&workflow_state, "work_items", "work_item_id")?;
    let workflow_artifacts = array_records(&workflow_state, "artifacts", "artifact_id")?;
    let workflow_reviews = array_records(&workflow_state, "reviews", "review_id")?;
    let workflow_bindings = array_records(
        &workflow_state,
        "workflow_node_session_bindings",
        "binding_id",
    )?;
    let workflow_dispatches =
        array_records(&workflow_state, "workflow_node_dispatches", "dispatch_id")?;
    let workflow_execution_attempts =
        optional_array_records(&workflow_state, "execution_attempts", "attempt_id")?;
    let workflow_chain_runs =
        optional_array_records(&workflow_state, "workflow_chain_runs", "chain_run_id")?;
    let workflow_execution_controls =
        optional_array_records(&workflow_state, "workflow_execution_controls", "control_id")?;
    let workflow_permission_requests =
        optional_array_records(&workflow_state, "permission_requests", "request_id")?;
    let workflow_capabilities = array_records(&workflow_state, "capabilities", "capability_id")?;
    let workflow_harness_resources =
        array_records(&workflow_state, "harness_resources", "resource_id")?;
    let workflow_audits = array_records(&workflow_state, "audit_events", "event_id")?;
    let supervisor_records = values_to_records(supervisor_actions, "action_id")?;
    let supervisor_orchestrator_session_records =
        values_to_records(supervisor_orchestrator_sessions, "run_id")?;
    let supervisor_orchestrator_audit_records =
        values_to_records(supervisor_orchestrator_audit_events, "event_id")?;

    let proposal_db_audits = merge_records(
        "project_proposal_audit_events",
        vec![
            database.proposal_audits,
            audit_records_for(&database.audits, "project_consultation_proposal"),
        ],
    )?;
    let authorization_db_audits = merge_records(
        "plan_authorization_audit_events",
        vec![
            database.authorization_audits,
            audit_records_for(&database.audits, "plan_authorization"),
        ],
    )?;
    let workflow_db_audits = audit_records_for(&database.audits, "workflow_state");
    let audit_db_records = proposal_db_audits
        .iter()
        .chain(authorization_db_audits.iter())
        .chain(workflow_db_audits.iter())
        .cloned()
        .collect::<Vec<_>>();
    let audit_json_records = proposal_audits
        .into_iter()
        .chain(authorization_audits)
        .chain(workflow_audits)
        .collect::<Vec<_>>();
    let db_unprojected_audits = database.audits.len().saturating_sub(audit_db_records.len());

    let mut tables = vec![
        reconcile_table("project_proposals", database.proposals, proposal_records),
        reconcile_table(
            "plan_authorizations",
            normalize_authorizations(database.authorizations)?,
            authorization_records,
        ),
        reconcile_table("projects", database.projects, workflow_projects),
        reconcile_table(
            "agent_adapters",
            database.agent_adapters,
            workflow_agent_adapters,
        ),
        reconcile_table("workflows", database.workflows, workflow_workflows),
        reconcile_table("workflow_nodes", database.nodes, workflow_nodes),
        reconcile_table("workflow_edges", database.edges, workflow_edges),
        reconcile_table("work_items", database.work_items, workflow_work_items),
        reconcile_table("workflow_artifacts", database.artifacts, workflow_artifacts),
        reconcile_table("workflow_reviews", database.reviews, workflow_reviews),
        reconcile_table(
            "workflow_node_session_bindings",
            database.bindings,
            workflow_bindings,
        ),
        reconcile_table(
            "workflow_node_dispatches",
            database.dispatches,
            workflow_dispatches,
        ),
        reconcile_table(
            "execution_attempts",
            database.execution_attempts,
            workflow_execution_attempts,
        ),
        reconcile_table(
            "workflow_chain_runs",
            database.chain_runs,
            workflow_chain_runs,
        ),
        reconcile_table(
            "workflow_execution_controls",
            database.execution_controls,
            workflow_execution_controls,
        ),
        reconcile_table(
            "permission_requests",
            database.permission_requests,
            workflow_permission_requests,
        ),
        reconcile_table("capabilities", database.capabilities, workflow_capabilities),
        reconcile_table(
            "harness_resources",
            database.harness_resources,
            workflow_harness_resources,
        ),
        reconcile_table(
            "supervisor_actions",
            normalize_supervisor_actions(database.supervisor_actions)?,
            supervisor_records,
        ),
        reconcile_table(
            "supervisor_orchestrator_sessions",
            database.supervisor_orchestrator_sessions,
            supervisor_orchestrator_session_records,
        ),
        reconcile_table(
            "supervisor_orchestrator_audit_events",
            database.supervisor_orchestrator_audit_events,
            supervisor_orchestrator_audit_records,
        ),
        reconcile_table_with_unprojected(
            "workflow_audit_events",
            audit_db_records,
            audit_json_records,
            db_unprojected_audits,
        ),
    ];
    tables.extend(m5c::reconcile_tables(
        &database.m5c,
        &config.workflow_state_path,
    )?);
    let status = if tables.iter().all(|table| {
        table.db_leading.is_empty()
            && table.json_leading.is_empty()
            && table.hash_mismatches.is_empty()
    }) {
        "green".to_string()
    } else {
        "not_green".to_string()
    };
    Ok(DbJsonReconciliationReport { status, tables })
}

fn replay_db_primary_projection(config: &DbPrimaryJsonProjectionConfig) -> Result<(), String> {
    let database = load_db_projection_data(config)?;
    let report = reconcile_db_vs_json(config)?;
    if report.has_json_leading_or_divergence() {
        return Err(report.fail_closed_reason());
    }
    let replace_db_primary_leading = report.has_db_leading();
    let timestamp_ms = crate::unix_timestamp_ms();
    let write_id = format!("db-primary-replay-{timestamp_ms}");
    let proposal_audits = merge_records(
        "project_proposal_audit_events",
        vec![
            database.proposal_audits.clone(),
            audit_records_for(&database.audits, "project_consultation_proposal"),
        ],
    )?;
    let authorization_audits = merge_records(
        "plan_authorization_audit_events",
        vec![
            database.authorization_audits.clone(),
            audit_records_for(&database.audits, "plan_authorization"),
        ],
    )?;
    crate::project_consultation_proposal_store::replay_db_primary_projection(
        &config.workflow_state_path,
        &database
            .proposals
            .iter()
            .map(|record| record.value.clone())
            .collect::<Vec<_>>(),
        &proposal_audits
            .iter()
            .map(|record| record.value.clone())
            .collect::<Vec<_>>(),
        replace_db_primary_leading,
        timestamp_ms,
        &write_id,
    )?;
    crate::plan_authorization_store::replay_db_primary_projection(
        &config.workflow_state_path,
        &normalize_authorizations(database.authorizations)?
            .into_iter()
            .map(|record| record.value)
            .collect::<Vec<_>>(),
        &authorization_audits
            .iter()
            .map(|record| record.value.clone())
            .collect::<Vec<_>>(),
        replace_db_primary_leading,
        timestamp_ms,
        &write_id,
    )?;
    replay_workflow_state_projection(
        &config.workflow_state_path,
        &database.projects,
        &database.agent_adapters,
        &database.workflows,
        &database.nodes,
        &database.edges,
        &database.work_items,
        &database.artifacts,
        &database.reviews,
        &database.bindings,
        &database.dispatches,
        &database.execution_attempts,
        &database.chain_runs,
        &database.execution_controls,
        &database.permission_requests,
        &database.capabilities,
        &database.harness_resources,
        &audit_records_for(&database.audits, "workflow_state"),
        replace_db_primary_leading,
    )?;
    crate::supervisor_action_controller::replay_db_primary_projection(
        &config.workflow_state_path,
        &normalize_supervisor_actions(database.supervisor_actions)?
            .into_iter()
            .map(|record| record.value)
            .collect::<Vec<_>>(),
        replace_db_primary_leading,
        &write_id,
    )?;
    crate::mcp::supervisor_orchestrator::replay_db_primary_projection(
        &config.workflow_state_path,
        &database
            .supervisor_orchestrator_sessions
            .iter()
            .map(|record| record.value.clone())
            .collect::<Vec<_>>(),
        &database
            .supervisor_orchestrator_audit_events
            .iter()
            .map(|record| record.value.clone())
            .collect::<Vec<_>>(),
        replace_db_primary_leading,
        &write_id,
    )?;
    m5c::replay_db_primary_projection(
        &config.workflow_state_path,
        &database.m5c,
        replace_db_primary_leading,
        timestamp_ms,
        &write_id,
    )?;
    Ok(())
}

/// A crash after the DB-primary commit and JSON projection, but before the
/// M2 projection checkpoint, leaves the named reference slice green at the
/// DB/JSON layer while its checkpoint is stale.  Startup repairs only that
/// exact, already-authoritative checkpoint: it re-reads the persisted JSON,
/// derives the same full aggregate DTO/hash as the SQLite port, and refuses
/// to advance anything if the source snapshot/event/receipt no longer match.
/// No JSON value is ever imported back into SQLite.
fn repair_m2_workflow_state_sidecar_checkpoint_after_startup(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<(), String> {
    let persisted_projection = crate::read_workflow_state_value(&config.workflow_state_path)?;
    let repository = WorkbenchSqliteRepository::open_confirmed(&config.repository_config())?;
    repository
        .with_immediate_transaction(
            "m2_workflow_state_sidecar_startup_checkpoint_recovery",
            None,
            |transaction| {
                let snapshots = {
                    let mut statement = transaction
                        .prepare(
                            "SELECT object_ref, object_revision, source_watermark, snapshot_hash
                             FROM current_snapshots
                             WHERE projector_id = 'workflow_projector'
                               AND object_ref LIKE 'workflow_state:%'
                             ORDER BY object_ref",
                        )
                        .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Sqlite)?;
                    let rows = statement
                        .query_map([], |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, String>(3)?,
                            ))
                        })
                        .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Sqlite)?;
                    rows
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Sqlite)?
                };
                let Some((object_ref, revision, source_watermark, stored_hash)) =
                    snapshots.into_iter().next()
                else {
                    return Ok(())
                        as Result<(), crate::workbench_sqlite_repository::RepositoryMutationError>;
                };
                if transaction
                    .query_row(
                        "SELECT COUNT(*) FROM current_snapshots
                         WHERE projector_id = 'workflow_projector'
                           AND object_ref LIKE 'workflow_state:%'",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Sqlite)?
                    != 1
                {
                    return Err(
                        crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                            "m2_workflow_state_startup_checkpoint_multiple_named_slices".to_string(),
                        ),
                    );
                }
                let (project_ref, workflow_id) =
                    m2_workflow_state_projection_identity_for_object_ref(
                        &object_ref,
                        &persisted_projection,
                    )
                    .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Message)?;
                let projection_snapshot = crate::workbench_sqlite_repository::
                    m2_workflow_state_sidecar_snapshot_from_projection(
                        &project_ref,
                        &workflow_id,
                        revision,
                        &persisted_projection,
                    )
                    .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Message)?;
                if projection_snapshot.object_ref != object_ref
                    || projection_snapshot.snapshot_hash != stored_hash
                {
                    return Err(
                        crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                            "m2_workflow_state_startup_checkpoint_projection_hash_mismatch".to_string(),
                        ),
                    );
                }
                let checkpoint: Option<(Option<String>, String, String)> = transaction
                    .query_row(
                        "SELECT last_event_id, source_watermark, status
                         FROM projection_checkpoints
                         WHERE projector_id = ?1 AND projector_version = ?2",
                        [
                            crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                            crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
                        ],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .optional()
                    .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Sqlite)?;
                if checkpoint.as_ref().is_some_and(|(last_event_id, watermark, status)| {
                    last_event_id.as_deref() == Some(source_watermark.as_str())
                        && watermark == &source_watermark
                        && status == "CAUGHT_UP"
                }) {
                    return Ok(())
                        as Result<(), crate::workbench_sqlite_repository::RepositoryMutationError>;
                }
                let receipt_id: String = transaction
                    .query_row(
                        "SELECT receipts.receipt_id
                         FROM events
                         JOIN command_receipts AS receipts
                           ON receipts.command_id = events.command_id
                         WHERE events.event_id = ?1
                         ORDER BY receipts.receipt_id
                         LIMIT 1",
                        [&source_watermark],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Sqlite)?
                    .ok_or_else(|| {
                        crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                            "m2_workflow_state_startup_checkpoint_source_receipt_missing".to_string(),
                        )
                    })?;
                crate::workbench_sqlite_repository::WorkflowStateSidecarRepositoryV1::new(
                    transaction,
                    crate::workbench_sqlite_repository::M2WorkflowStateSidecarConsumerId::StartupCheckpointRecovery,
                )
                .record_projection_checkpoint(
                    &object_ref,
                    revision,
                    &source_watermark,
                    &receipt_id,
                    &projection_snapshot.snapshot_hash,
                    crate::unix_timestamp_ms(),
                )?;
                Ok(()) as Result<(), crate::workbench_sqlite_repository::RepositoryMutationError>
            },
        )
        .map_err(|error| format!("m2_workflow_state_startup_checkpoint_recovery:{error}"))?;
    Ok(())
}

fn m2_workflow_state_projection_identity_for_object_ref(
    object_ref: &str,
    projection: &Value,
) -> Result<(String, String), String> {
    let remainder = object_ref
        .strip_prefix("workflow_state:")
        .ok_or_else(|| "m2_workflow_state_startup_checkpoint_object_ref_invalid".to_string())?;
    let workflows = projection
        .get("workflows")
        .and_then(Value::as_array)
        .ok_or_else(|| "m2_workflow_state_startup_checkpoint_workflows_missing".to_string())?;
    let candidates = workflows
        .iter()
        .filter_map(|workflow| {
            let workflow_id = workflow.get("workflow_id").and_then(Value::as_str)?;
            let suffix = format!(":{workflow_id}");
            remainder
                .strip_suffix(&suffix)
                .filter(|project_ref| !project_ref.trim().is_empty())
                .map(|project_ref| (project_ref.to_string(), workflow_id.to_string()))
        })
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [identity] => Ok(identity.clone()),
        [] => Err("m2_workflow_state_startup_checkpoint_object_ref_unjoinable".to_string()),
        _ => Err("m2_workflow_state_startup_checkpoint_object_ref_ambiguous".to_string()),
    }
}

fn has_stable_startup_mode_audit(value: &Value, db_path_hash: &str) -> bool {
    value
        .get("audit_events")
        .and_then(Value::as_array)
        .is_some_and(|audits| {
            audits.iter().any(|audit| {
                audit.get("event_type").and_then(Value::as_str)
                    == Some("storage_mode_initialized")
                    && audit.get("target_ref").and_then(Value::as_str) == Some(db_path_hash)
                    && audit.get("actor_ref").and_then(Value::as_str)
                        == Some("workbench_storage_mode")
                    && audit.get("source_kind").and_then(Value::as_str)
                        == Some("workspace_state")
                    && audit.get("permission_level").and_then(Value::as_str)
                        == Some("system_runtime")
                    && audit.get("before_state").and_then(Value::as_str)
                        == Some("db_primary_json_projection")
                    && audit.get("after_state").and_then(Value::as_str)
                        == Some("db_primary_json_projection")
            })
        })
}

fn append_startup_mode_audit(
    config: &DbPrimaryJsonProjectionConfig,
    replayed_db_primary_projection: bool,
) -> Result<(), String> {
    let mut value = crate::read_workflow_state_value(&config.workflow_state_path)?;
    let db_path_hash = config.db_path_hash();
    // A green DB-primary restart is a read-only recovery check.  The original
    // initialization audit is durable evidence, but writing another one on
    // every restart changes both the projection and DB without a business
    // transition.  A DB-leading replay remains explicitly auditable.
    if !replayed_db_primary_projection && has_stable_startup_mode_audit(&value, &db_path_hash) {
        return Ok(());
    }
    let timestamp = crate::unix_timestamp_string();
    let event_id = crate::workflow_audit::audit_event_identity(
        "storage-mode-startup",
        &db_path_hash,
        &timestamp,
    );
    let event = json!({
        "event_id": event_id,
        "event_type": if replayed_db_primary_projection { "storage_mode_projection_replayed" } else { "storage_mode_initialized" },
        "target_ref": db_path_hash,
        "actor_ref": "workbench_storage_mode",
        "source_kind": "workspace_state",
        "permission_level": "system_runtime",
        "before_state": "db_primary_json_projection",
        "after_state": "db_primary_json_projection",
        "created_at": timestamp,
        "reason": if replayed_db_primary_projection { "检测到 DB 领先，已重放 JSON 投影并完成启动对账。" } else { "已完成 DB 主写与 JSON 投影启动对账。" }
    });
    let audit = RepositoryAuditEntry {
        event_id: event_id.clone(),
        target_kind: "workflow_state".to_string(),
        target_id: event_id.clone(),
        payload: event.clone(),
    };
    WorkbenchSqliteRepository::open_confirmed(&config.repository_config())?
        .append_audit(&audit, None)?;
    let audits = value
        .get_mut("audit_events")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "storage_mode_workflow_audit_array_required".to_string())?;
    audits.push(event);
    value["updated_at"] = Value::String(timestamp.clone());
    crate::backup_workflow_state_file(&config.workflow_state_path, &timestamp)?;
    crate::write_validated_workflow_state(&config.workflow_state_path, &value)
}

fn load_db_projection_data(
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<DbProjectionData, String> {
    let connection = Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("db_json_reconcile_open_read_only_failed:{error}"))?;
    Ok(DbProjectionData {
        proposals: query_records(
            &connection,
            "SELECT proposal_id, record_hash, record_json FROM project_proposals",
        )?,
        proposal_audits: query_records(
            &connection,
            "SELECT audit_event_id, record_hash, record_json FROM project_proposal_audit_events",
        )?,
        authorizations: query_records(
            &connection,
            "SELECT authorization_id, record_hash, record_json FROM plan_authorizations",
        )?,
        authorization_audits: query_records(
            &connection,
            "SELECT audit_event_id, record_hash, record_json FROM plan_authorization_audit_events",
        )?,
        projects: query_records(
            &connection,
            "SELECT project_id, record_hash, record_json FROM projects",
        )?,
        agent_adapters: query_records(
            &connection,
            "SELECT adapter_id, record_hash, record_json FROM agent_adapters",
        )?,
        workflows: query_records(
            &connection,
            "SELECT workflow_id, record_hash, record_json FROM workflows",
        )?,
        nodes: query_records(
            &connection,
            "SELECT node_id, record_hash, record_json FROM workflow_nodes",
        )?,
        edges: query_records(
            &connection,
            "SELECT edge_id, record_hash, record_json FROM workflow_edges",
        )?,
        work_items: query_records(
            &connection,
            "SELECT work_item_id, record_hash, record_json FROM work_items",
        )?,
        artifacts: query_records(
            &connection,
            "SELECT artifact_id, record_hash, record_json FROM workflow_artifacts",
        )?,
        reviews: query_records(
            &connection,
            "SELECT review_id, record_hash, record_json FROM workflow_reviews",
        )?,
        bindings: query_records(
            &connection,
            "SELECT binding_id, record_hash, record_json FROM workflow_node_session_bindings",
        )?,
        dispatches: query_records(
            &connection,
            "SELECT dispatch_id, record_hash, record_json FROM workflow_node_dispatches",
        )?,
        execution_attempts: query_records(
            &connection,
            "SELECT attempt_id, record_hash, record_json FROM execution_attempts",
        )?,
        chain_runs: query_records(
            &connection,
            "SELECT chain_run_id, record_hash, record_json FROM workflow_chain_runs",
        )?,
        execution_controls: query_records(
            &connection,
            "SELECT control_id, record_hash, record_json FROM workflow_execution_controls",
        )?,
        permission_requests: query_records(
            &connection,
            "SELECT request_id, record_hash, record_json FROM permission_requests",
        )?,
        capabilities: query_records(
            &connection,
            "SELECT capability_id, record_hash, record_json FROM capabilities",
        )?,
        harness_resources: query_records(
            &connection,
            "SELECT resource_id, record_hash, record_json FROM harness_resources",
        )?,
        supervisor_actions: query_records(
            &connection,
            "SELECT action_id, record_hash, record_json FROM supervisor_actions",
        )?,
        supervisor_orchestrator_sessions: query_records(
            &connection,
            "SELECT run_id, record_hash, record_json FROM supervisor_orchestrator_sessions",
        )?,
        supervisor_orchestrator_audit_events: query_records(
            &connection,
            "SELECT event_id, record_hash, record_json FROM supervisor_orchestrator_audit_events",
        )?,
        m5c: m5c::load_db_projection_data(&connection)?,
        audits: query_audit_records(&connection)?,
    })
}

fn query_records(connection: &Connection, sql: &str) -> Result<Vec<DbRecord>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("db_json_reconcile_prepare_failed:{error}"))?;
    let mut rows = statement
        .query([])
        .map_err(|error| format!("db_json_reconcile_query_failed:{error}"))?;
    let mut records = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| format!("db_json_reconcile_row_failed:{error}"))?
    {
        let natural_key: String = row
            .get(0)
            .map_err(|error| format!("db_json_reconcile_key_failed:{error}"))?;
        let stored_record_hash: String = row
            .get(1)
            .map_err(|error| format!("db_json_reconcile_hash_failed:{error}"))?;
        let record_json: String = row
            .get(2)
            .map_err(|error| format!("db_json_reconcile_json_failed:{error}"))?;
        let value = serde_json::from_str(&record_json)
            .map_err(|error| format!("db_json_reconcile_record_parse_failed:{error}"))?;
        validate_stored_record_hash(&stored_record_hash, &value)?;
        records.push(DbRecord {
            natural_key,
            record_hash: record_hash(&value)?,
            value,
        });
    }
    Ok(records)
}

fn query_audit_records(connection: &Connection) -> Result<Vec<DbAuditRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT event_id, target_kind, target_id, record_hash, record_json FROM workflow_audit_events",
        )
        .map_err(|error| format!("db_json_reconcile_audit_prepare_failed:{error}"))?;
    let mut rows = statement
        .query([])
        .map_err(|error| format!("db_json_reconcile_audit_query_failed:{error}"))?;
    let mut records = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| format!("db_json_reconcile_audit_row_failed:{error}"))?
    {
        let natural_key: String = row
            .get(0)
            .map_err(|error| format!("db_json_reconcile_audit_key_failed:{error}"))?;
        let target_kind: Option<String> = row
            .get(1)
            .map_err(|error| format!("db_json_reconcile_audit_target_kind_failed:{error}"))?;
        let _target_id: Option<String> = row
            .get(2)
            .map_err(|error| format!("db_json_reconcile_audit_target_id_failed:{error}"))?;
        let stored_record_hash: String = row
            .get(3)
            .map_err(|error| format!("db_json_reconcile_audit_hash_failed:{error}"))?;
        let record_json: String = row
            .get(4)
            .map_err(|error| format!("db_json_reconcile_audit_json_failed:{error}"))?;
        let value = serde_json::from_str(&record_json)
            .map_err(|error| format!("db_json_reconcile_audit_parse_failed:{error}"))?;
        validate_stored_record_hash(&stored_record_hash, &value)?;
        records.push(DbAuditRecord {
            record: DbRecord {
                natural_key,
                record_hash: record_hash(&value)?,
                value,
            },
            // M3 imported workflow audit rows before the repository introduced target_kind.
            // Those rows are the main workflow projection, not unclassified sidecar records.
            target_kind: target_kind
                .filter(|kind| !kind.trim().is_empty())
                .unwrap_or_else(|| "workflow_state".to_string()),
        });
    }
    Ok(records)
}

fn values_to_records(values: Vec<Value>, key_field: &str) -> Result<Vec<DbRecord>, String> {
    values
        .into_iter()
        .map(|value| {
            let natural_key = value
                .get(key_field)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| format!("db_json_reconcile_key_required:{key_field}"))?
                .to_string();
            let record_hash = record_hash(&value)?;
            Ok(DbRecord {
                natural_key,
                record_hash,
                value,
            })
        })
        .collect()
}

fn array_records(root: &Value, array_name: &str, key_field: &str) -> Result<Vec<DbRecord>, String> {
    let values = root
        .get(array_name)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("db_json_reconcile_array_required:{array_name}"))?
        .clone();
    values_to_records(values, key_field)
}

// These four arrays were introduced after the original v0 fixture shape. Missing means an
// unmaterialized empty collection, not a divergent record; replay materializes it only when DB
// has a row to project.
fn optional_array_records(
    root: &Value,
    array_name: &str,
    key_field: &str,
) -> Result<Vec<DbRecord>, String> {
    match root.get(array_name) {
        None => Ok(vec![]),
        Some(Value::Array(values)) => values_to_records(values.clone(), key_field),
        Some(_) => Err(format!("db_json_reconcile_array_required:{array_name}")),
    }
}

fn audit_records_for(audits: &[DbAuditRecord], target_kind: &str) -> Vec<DbRecord> {
    audits
        .iter()
        .filter(|audit| audit.target_kind == target_kind)
        .map(|audit| audit.record.clone())
        .collect()
}

fn merge_records(table_name: &str, groups: Vec<Vec<DbRecord>>) -> Result<Vec<DbRecord>, String> {
    let mut merged = BTreeMap::<String, DbRecord>::new();
    for record in groups.into_iter().flatten() {
        match merged.get(&record.natural_key) {
            Some(existing) if existing.record_hash != record.record_hash => {
                return Err(format!(
                    "db_json_reconcile_duplicate_hash_mismatch:{table_name}:{}",
                    record.natural_key
                ));
            }
            Some(_) => {}
            None => {
                merged.insert(record.natural_key.clone(), record);
            }
        }
    }
    Ok(merged.into_values().collect())
}

fn normalize_authorizations(records: Vec<DbRecord>) -> Result<Vec<DbRecord>, String> {
    records
        .into_iter()
        .map(|mut record| {
            let object = record.value.as_object_mut().ok_or_else(|| {
                format!(
                    "db_json_reconcile_authorization_object_required:{}",
                    record.natural_key
                )
            })?;
            let proposal_id = object.remove("proposal_id");
            object.remove("revision");
            if let Some(proposal_id) = proposal_id {
                object.insert("source_proposal_id".to_string(), proposal_id);
            }
            record.record_hash = record_hash(&record.value)?;
            Ok(record)
        })
        .collect()
}

fn normalize_supervisor_actions(records: Vec<DbRecord>) -> Result<Vec<DbRecord>, String> {
    records
        .into_iter()
        .map(|mut record| {
            let result = record.value.get("result").cloned();
            let object = record.value.as_object_mut().ok_or_else(|| {
                format!(
                    "db_json_reconcile_supervisor_action_object_required:{}",
                    record.natural_key
                )
            })?;
            if let Some(Value::Object(result)) = result {
                for (key, value) in result {
                    if key == "status" {
                        object.insert("execution_status".to_string(), value);
                    } else {
                        object.insert(key, value);
                    }
                }
            }
            object.remove("result");
            record.record_hash = record_hash(&record.value)?;
            Ok(record)
        })
        .collect()
}

fn reconcile_table(
    table_name: &str,
    db_records: Vec<DbRecord>,
    json_records: Vec<DbRecord>,
) -> DbJsonTableReconciliation {
    reconcile_table_with_unprojected(table_name, db_records, json_records, 0)
}

fn reconcile_table_with_unprojected(
    table_name: &str,
    db_records: Vec<DbRecord>,
    json_records: Vec<DbRecord>,
    db_unprojected_count: usize,
) -> DbJsonTableReconciliation {
    let db_count = db_records.len();
    let json_count = json_records.len();
    let db = db_records
        .into_iter()
        .map(|record| (record.natural_key.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let json = json_records
        .into_iter()
        .map(|record| (record.natural_key.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let mut matched_count = 0;
    let mut db_leading = Vec::new();
    let mut json_leading = Vec::new();
    let mut hash_mismatches = Vec::new();

    for (key, db_record) in &db {
        match json.get(key) {
            None => db_leading.push(key.clone()),
            Some(json_record) if db_record.record_hash == json_record.record_hash => {
                matched_count += 1;
            }
            Some(json_record) => {
                match compare_record_freshness(&db_record.value, &json_record.value) {
                    Some(std::cmp::Ordering::Greater) => db_leading.push(key.clone()),
                    Some(std::cmp::Ordering::Less) => json_leading.push(key.clone()),
                    _ => hash_mismatches.push(key.clone()),
                }
            }
        }
    }
    for key in json.keys() {
        if !db.contains_key(key) {
            json_leading.push(key.clone());
        }
    }
    DbJsonTableReconciliation {
        table_name: table_name.to_string(),
        db_count,
        json_count,
        matched_count,
        db_leading,
        json_leading,
        hash_mismatches,
        db_unprojected_count,
    }
}

fn compare_record_freshness(db: &Value, json: &Value) -> Option<std::cmp::Ordering> {
    let db_revision = db.get("workflow_revision_after").and_then(Value::as_i64);
    let json_revision = json.get("workflow_revision_after").and_then(Value::as_i64);
    match (db_revision, json_revision) {
        (Some(db_revision), Some(json_revision)) if db_revision != json_revision => {
            return Some(db_revision.cmp(&json_revision));
        }
        (Some(_), None) => return Some(std::cmp::Ordering::Greater),
        (None, Some(_)) => return Some(std::cmp::Ordering::Less),
        _ => {}
    }
    // Edges and some legacy artifacts carry no temporal sequence. Do not invent one: a
    // divergent hash without an explicit order remains a fail-closed hash_mismatch.
    let db_order = record_order(db)?;
    let json_order = record_order(json)?;
    Some(db_order.cmp(&json_order))
}

fn record_order(value: &Value) -> Option<String> {
    for key in [
        "workflow_revision_after",
        "updated_at_ms",
        "created_at_ms",
        "updated_at",
        "created_at",
        "completed_at",
        "finished_at",
        "confirmed_at",
        "started_at",
    ] {
        if let Some(number) = value.get(key).and_then(Value::as_i64) {
            return Some(format!("n:{number:020}"));
        }
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            if !text.trim().is_empty() {
                return Some(format!("s:{text}"));
            }
        }
    }
    if let Some(request) = value.get("request") {
        for key in ["created_at", "updated_at"] {
            if let Some(number) = request.get(key).and_then(Value::as_i64) {
                return Some(format!("n:{number:020}"));
            }
            if let Some(text) = request.get(key).and_then(Value::as_str) {
                if !text.trim().is_empty() {
                    return Some(format!("s:{text}"));
                }
            }
        }
    }
    None
}

fn replay_workflow_state_projection(
    workflow_state_path: &Path,
    projects: &[DbRecord],
    agent_adapters: &[DbRecord],
    workflows: &[DbRecord],
    nodes: &[DbRecord],
    edges: &[DbRecord],
    work_items: &[DbRecord],
    artifacts: &[DbRecord],
    reviews: &[DbRecord],
    bindings: &[DbRecord],
    dispatches: &[DbRecord],
    execution_attempts: &[DbRecord],
    chain_runs: &[DbRecord],
    execution_controls: &[DbRecord],
    permission_requests: &[DbRecord],
    capabilities: &[DbRecord],
    harness_resources: &[DbRecord],
    audits: &[DbRecord],
    replace_db_primary_leading: bool,
) -> Result<usize, String> {
    let mut state = crate::read_workflow_state_value(workflow_state_path)?;
    let mut changes = 0;
    changes += replay_array_records(
        &mut state,
        "projects",
        "project_id",
        projects,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "agent_adapters",
        "adapter_id",
        agent_adapters,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "workflows",
        "workflow_id",
        workflows,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "nodes",
        "node_id",
        nodes,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "edges",
        "edge_id",
        edges,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "work_items",
        "work_item_id",
        work_items,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "artifacts",
        "artifact_id",
        artifacts,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "reviews",
        "review_id",
        reviews,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "workflow_node_session_bindings",
        "binding_id",
        bindings,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "workflow_node_dispatches",
        "dispatch_id",
        dispatches,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "execution_attempts",
        "attempt_id",
        execution_attempts,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "workflow_chain_runs",
        "chain_run_id",
        chain_runs,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "workflow_execution_controls",
        "control_id",
        execution_controls,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "permission_requests",
        "request_id",
        permission_requests,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "capabilities",
        "capability_id",
        capabilities,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "harness_resources",
        "resource_id",
        harness_resources,
        replace_db_primary_leading,
    )?;
    changes += replay_array_records(
        &mut state,
        "audit_events",
        "event_id",
        audits,
        replace_db_primary_leading,
    )?;
    if changes > 0 {
        let timestamp = crate::unix_timestamp_string();
        state["updated_at"] = Value::String(timestamp.clone());
        crate::backup_workflow_state_file(workflow_state_path, &timestamp)?;
        crate::write_validated_workflow_state(workflow_state_path, &state)?;
    }
    Ok(changes)
}

fn replay_array_records(
    state: &mut Value,
    array_name: &str,
    key_field: &str,
    records: &[DbRecord],
    replace_db_primary_leading: bool,
) -> Result<usize, String> {
    if state.get(array_name).is_none() {
        state[array_name] = Value::Array(vec![]);
    }
    let array = state
        .get_mut(array_name)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("db_json_projection_array_required:{array_name}"))?;
    let mut changes = 0;
    for record in records {
        let index = array.iter().position(|value| {
            value.get(key_field).and_then(Value::as_str) == Some(record.natural_key.as_str())
        });
        match index {
            None => {
                array.push(record.value.clone());
                changes += 1;
            }
            Some(index) => {
                let current_hash = record_hash(&array[index])?;
                if current_hash != record.record_hash {
                    if !replace_db_primary_leading {
                        return Err(format!(
                            "db_json_projection_hash_mismatch:{array_name}:{}",
                            record.natural_key
                        ));
                    }
                    array[index] = record.value.clone();
                    changes += 1;
                }
            }
        }
    }
    Ok(changes)
}

fn record_hash(value: &Value) -> Result<String, String> {
    Ok(crate::workbench_sqlite_importer::canonical_json_hash(value))
}

fn validate_stored_record_hash(stored_hash: &str, value: &Value) -> Result<(), String> {
    let repository_hash = sha256_hex(
        &serde_json::to_string(value)
            .map_err(|error| format!("db_json_reconcile_record_serialize_failed:{error}"))?,
    );
    let canonical_hash = crate::workbench_sqlite_importer::canonical_json_hash(value);
    if stored_hash != repository_hash && stored_hash != canonical_hash {
        return Err("db_json_reconcile_stored_record_hash_invalid".to_string());
    }
    Ok(())
}

fn resolve_storage_mode(workflow_state_path: &Path) -> StorageMode {
    let config_path = match storage_mode_path(workflow_state_path) {
        Ok(path) => path,
        Err(reason) => return json_only(reason),
    };
    if !config_path.exists() {
        return json_only(format!(
            "storage_mode_config_missing:{}",
            config_path.display()
        ));
    }
    let text = match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(error) => {
            return json_only(format!(
                "storage_mode_config_read_failed:{}:{error}",
                config_path.display()
            ));
        }
    };
    let file: StorageModeFileV1 = match serde_json::from_str(&text) {
        Ok(file) => file,
        Err(error) => {
            return json_only(format!(
                "storage_mode_config_parse_failed:{}:{error}",
                config_path.display()
            ));
        }
    };
    if file.schema_version != STORAGE_MODE_SCHEMA_VERSION {
        return json_only(format!(
            "storage_mode_schema_version_invalid:{}",
            file.schema_version
        ));
    }
    if file.mode == JSON_ONLY {
        return json_only("storage_mode_config_json_only".to_string());
    }
    if file.mode != DB_PRIMARY_JSON_PROJECTION {
        return json_only(format!("storage_mode_value_invalid:{}", file.mode));
    }
    let Some(configured_workflow_state_path) = file.workflow_state_path else {
        return json_only("storage_mode_workflow_state_path_required".to_string());
    };
    let Some(configured_confirmed_workflow_state_path) = file.confirmed_workflow_state_path else {
        return json_only("storage_mode_confirmed_workflow_state_path_required".to_string());
    };
    let Some(configured_db_path) = file.db_path else {
        return json_only("storage_mode_db_path_required".to_string());
    };
    let Some(configured_confirmed_db_path) = file.confirmed_db_path else {
        return json_only("storage_mode_confirmed_db_path_required".to_string());
    };
    let config = DbPrimaryJsonProjectionConfig {
        workflow_state_path: configured_workflow_state_path,
        confirmed_workflow_state_path: configured_confirmed_workflow_state_path,
        db_path: configured_db_path,
        confirmed_db_path: configured_confirmed_db_path,
        denied_path_markers: file.denied_path_markers,
    };
    match validate_db_primary_config(workflow_state_path, &config) {
        Ok(()) => StorageMode::DbPrimaryJsonProjection(config),
        Err(reason) => json_only(reason),
    }
}

fn validate_db_primary_config(
    actual_workflow_state_path: &Path,
    config: &DbPrimaryJsonProjectionConfig,
) -> Result<(), String> {
    if config.workflow_state_path != actual_workflow_state_path
        || config.confirmed_workflow_state_path != actual_workflow_state_path
    {
        return Err(format!(
            "storage_mode_workflow_state_path_mismatch:expected={}:configured={}:confirmed={}",
            actual_workflow_state_path.display(),
            config.workflow_state_path.display(),
            config.confirmed_workflow_state_path.display()
        ));
    }
    validate_clean_canonical_path("workflow_state_path", actual_workflow_state_path)?;
    if config.db_path != config.confirmed_db_path {
        return Err(format!(
            "storage_mode_confirmed_db_path_mismatch:db={}:confirmed={}",
            config.db_path.display(),
            config.confirmed_db_path.display()
        ));
    }
    validate_clean_canonical_path("db_path", &config.db_path)?;
    let mut denied = CONFIRMED_DB_DENIED_PATH_MARKERS
        .iter()
        .map(|marker| (*marker).to_string())
        .collect::<Vec<_>>();
    denied.extend(config.denied_path_markers.iter().cloned());
    let normalized = config.db_path.to_string_lossy().to_ascii_lowercase();
    if denied
        .iter()
        .map(|marker| marker.trim().to_ascii_lowercase())
        .filter(|marker| !marker.is_empty())
        .any(|marker| normalized.contains(&marker))
    {
        return Err(format!(
            "storage_mode_db_path_denied_marker:{}",
            config.db_path.display()
        ));
    }
    // An acceptance R4 process has one legal storage authority: its active
    // isolated root.  Canonicality alone is insufficient because another
    // clean temporary DB would otherwise be accepted as DB-primary.
    let active_profile = crate::acceptance_runtime_profile::active_paths()?;
    validate_active_r4_root_binding(actual_workflow_state_path, config, active_profile.as_ref())?;
    Ok(())
}

fn validate_active_r4_root_binding(
    actual_workflow_state_path: &Path,
    config: &DbPrimaryJsonProjectionConfig,
    active_profile: Option<&crate::acceptance_runtime_profile::RuntimePaths>,
) -> Result<(), String> {
    let Some(paths) = active_profile else {
        return Ok(());
    };
    if actual_workflow_state_path != paths.workflow_state_path {
        return Err(format!(
            "storage_mode_r4_workflow_state_path_mismatch:expected={}:actual={}",
            paths.workflow_state_path.display(),
            actual_workflow_state_path.display()
        ));
    }
    let expected_mode_path = paths
        .root
        .join("runtime-artifacts")
        .join(STORAGE_MODE_FILE_NAME);
    if storage_mode_path(actual_workflow_state_path)? != expected_mode_path {
        return Err(format!(
            "storage_mode_r4_config_path_mismatch:expected={}:actual={}",
            expected_mode_path.display(),
            storage_mode_path(actual_workflow_state_path)?.display()
        ));
    }
    let expected_db_path = paths
        .root
        .join("runtime-artifacts")
        .join("workbench.sqlite");
    if config.db_path != expected_db_path || config.confirmed_db_path != expected_db_path {
        return Err(format!(
            "storage_mode_r4_db_path_mismatch:expected={}:configured={}:confirmed={}",
            expected_db_path.display(),
            config.db_path.display(),
            config.confirmed_db_path.display()
        ));
    }
    Ok(())
}

fn validate_clean_canonical_path(label: &str, path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!(
            "storage_mode_absolute_path_required:{label}:{}",
            path.display()
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(format!(
            "storage_mode_path_must_be_clean:{label}:{}",
            path.display()
        ));
    }
    let canonical = canonicalize_existing_or_parent(path)?;
    if canonical != path {
        return Err(format!(
            "storage_mode_path_must_be_canonical:{label}:expected={}:actual={}",
            canonical.display(),
            path.display()
        ));
    }
    Ok(())
}

fn canonicalize_existing_or_parent(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|error| {
            format!(
                "storage_mode_canonicalize_failed:{}:{error}",
                path.display()
            )
        });
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("storage_mode_parent_required:{}", path.display()))?;
    let name = path
        .file_name()
        .ok_or_else(|| format!("storage_mode_file_name_required:{}", path.display()))?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "storage_mode_parent_canonicalize_failed:{}:{error}",
            parent.display()
        )
    })?;
    Ok(canonical_parent.join(name))
}

fn json_only(reason: String) -> StorageMode {
    StorageMode::JsonOnly { reason }
}

#[cfg(test)]
pub(crate) fn clear_storage_mode_cache_for_tests() {
    mode_cache()
        .lock()
        .expect("storage mode cache lock")
        .clear();
    health_cache()
        .lock()
        .expect("storage mode health lock")
        .clear();
    *degradation_audit_recorded()
        .lock()
        .expect("storage mode degradation audit lock") = false;
}

#[cfg(test)]
pub(crate) fn clear_storage_mode_cache_for_path_for_tests(workflow_state_path: &Path) {
    mode_cache()
        .lock()
        .expect("storage mode cache lock")
        .remove(workflow_state_path);
    health_cache()
        .lock()
        .expect("storage mode health lock")
        .remove(workflow_state_path);
}

#[cfg(test)]
pub(crate) fn storage_mode_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use serde_json::{json, Value};
    use std::cell::Cell;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Barrier, Mutex};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn test_lock() -> &'static Mutex<()> {
        storage_mode_test_lock()
    }

    struct DbPrimaryFixture {
        root: PathBuf,
        state_path: PathBuf,
        project_root: String,
        project_id: String,
        workflow_id: String,
        work_item_id: String,
        config: DbPrimaryJsonProjectionConfig,
    }

    impl Drop for DbPrimaryFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct JsonFallbackSupervisorAdapter {
        executions: Cell<usize>,
    }

    impl crate::supervisor_action_controller::SupervisorActionAdapter
        for JsonFallbackSupervisorAdapter
    {
        fn supports(
            &self,
            _action: &crate::supervisor_action_protocol::SupervisorActionKind,
        ) -> bool {
            true
        }

        fn execute(
            &self,
            _action: &crate::supervisor_action_controller::AuthorizedSupervisorAction,
        ) -> Result<crate::supervisor_action_controller::SupervisorActionAdapterResult, String>
        {
            self.executions.set(self.executions.get() + 1);
            Ok(
                crate::supervisor_action_controller::SupervisorActionAdapterResult {
                    status: "waiting_worker".to_string(),
                    summary: "M5-A blocked JSON fallback supervisor action".to_string(),
                    worker_id: Some("worker:m5a:blocked-json-fallback".to_string()),
                    adapter_id: "m5a-json-fallback-adapter".to_string(),
                    evidence_present: false,
                    dispatch_ref: Some("dispatch:m5a:blocked-json-fallback".to_string()),
                    readback_ref: None,
                    audit_ref: Some("audit:m5a:blocked-json-fallback".to_string()),
                },
            )
        }
    }

    fn fresh_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "m5a-db-primary-{label}-{}-{}",
            crate::unix_timestamp_nanos(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).expect("create temp fixture root");
        fs::canonicalize(root).expect("canonical temp fixture root")
    }

    fn r4_runtime_paths(root: PathBuf) -> crate::acceptance_runtime_profile::RuntimePaths {
        let app_data_root = root.join("app-data");
        crate::acceptance_runtime_profile::RuntimePaths {
            index_path: root.join("fixture/codex-index.json"),
            tasks_path: root.join("fixture/tasks.md"),
            project_root: root.join("fixture/SYN R4 ISOLATED ACCEPTANCE syn-r4-0123456789abcdef"),
            workflow_state_path: root.join("workflow-state/workflow-state.v0.json"),
            app_data_root: app_data_root.clone(),
            vault_root: app_data_root.join("knowledge-vault"),
            recovery_backups_root: root.join("app-data/knowledge-workspace-recovery"),
            canvas_root: root.join("app-data/canvas-v1"),
            codex_db_path: root.join("codex-db/state.sqlite"),
            app_log_dir: root.join("logs"),
            root,
        }
    }

    fn r4_db_primary_config(
        paths: &crate::acceptance_runtime_profile::RuntimePaths,
    ) -> DbPrimaryJsonProjectionConfig {
        let db_path = paths.root.join("runtime-artifacts/workbench.sqlite");
        DbPrimaryJsonProjectionConfig {
            workflow_state_path: paths.workflow_state_path.clone(),
            confirmed_workflow_state_path: paths.workflow_state_path.clone(),
            db_path: db_path.clone(),
            confirmed_db_path: db_path,
            denied_path_markers: vec![],
        }
    }

    #[test]
    fn m2_t2_r4_db_primary_accepts_exact_active_root_binding() {
        let paths = r4_runtime_paths(PathBuf::from("/tmp/syn-r4-acceptance-bound-root"));
        let config = r4_db_primary_config(&paths);
        validate_active_r4_root_binding(&paths.workflow_state_path, &config, Some(&paths))
            .expect("only the exact active R4 root may be DB-primary");
    }

    #[test]
    fn m2_t2_r4_db_primary_rejects_foreign_or_wrong_in_root_paths() {
        let paths = r4_runtime_paths(PathBuf::from("/tmp/syn-r4-acceptance-bound-root"));
        let mut foreign = r4_db_primary_config(&paths);
        foreign.db_path = PathBuf::from("/tmp/other-clean-root/workbench.sqlite");
        foreign.confirmed_db_path = foreign.db_path.clone();
        assert!(validate_active_r4_root_binding(
            &paths.workflow_state_path,
            &foreign,
            Some(&paths)
        )
        .expect_err("foreign canonical DB must be rejected")
        .starts_with("storage_mode_r4_db_path_mismatch"));

        let mut wrong_name = r4_db_primary_config(&paths);
        wrong_name.db_path = paths.root.join("runtime-artifacts/other.sqlite");
        wrong_name.confirmed_db_path = wrong_name.db_path.clone();
        assert!(validate_active_r4_root_binding(
            &paths.workflow_state_path,
            &wrong_name,
            Some(&paths)
        )
        .expect_err("wrong in-root DB name must be rejected")
        .starts_with("storage_mode_r4_db_path_mismatch"));

        let mut wrong_state = r4_db_primary_config(&paths);
        wrong_state.workflow_state_path = paths.root.join("workflow-state/other.json");
        wrong_state.confirmed_workflow_state_path = wrong_state.workflow_state_path.clone();
        assert!(validate_active_r4_root_binding(
            &wrong_state.workflow_state_path,
            &wrong_state,
            Some(&paths),
        )
        .expect_err("wrong active workflow state must be rejected")
        .starts_with("storage_mode_r4_workflow_state_path_mismatch"));
    }

    #[test]
    fn m2_t2_r4_db_primary_normal_mode_remains_unbound() {
        let paths = r4_runtime_paths(PathBuf::from("/tmp/syn-r4-acceptance-bound-root"));
        let mut config = r4_db_primary_config(&paths);
        config.db_path = PathBuf::from("/tmp/ordinary-workbench.sqlite");
        config.confirmed_db_path = config.db_path.clone();
        validate_active_r4_root_binding(&paths.workflow_state_path, &config, None)
            .expect("normal mode must not inherit R4 root constraints");
    }

    fn project_record(project_root: &str) -> crate::ProjectRecord {
        crate::ProjectRecord {
            project_root: project_root.to_string(),
            name: "M5-A DB primary fixture".to_string(),
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
        }
    }

    fn bootstrap_json_state(root: &Path, project_root: &str) -> (PathBuf, String, String, String) {
        let state_path = root.join("workflow-state").join("workflow-state.v0.json");
        fs::create_dir_all(state_path.parent().expect("state parent")).expect("state parent");
        fs::create_dir_all(project_root).expect("fixture project root");
        crate::bootstrap_project_workflow_at(&state_path, &project_record(project_root))
            .expect("bootstrap workflow state");
        crate::create_task_draft_at(
            &state_path,
            &crate::TaskDraftRequest {
                project_root: project_root.to_string(),
                title: "M5-A DB primary task".to_string(),
                objective: "exercise DB primary JSON projection".to_string(),
                assigned_role: Some("codex-dev".to_string()),
            },
        )
        .expect("create task draft");
        let value = crate::read_workflow_state_value(&state_path).expect("read task draft state");
        let work_item_id = value["work_items"][0]["work_item_id"]
            .as_str()
            .expect("work item id")
            .to_string();
        crate::update_work_item_state_at(
            &state_path,
            &crate::WorkItemStateUpdateRequest {
                project_root: project_root.to_string(),
                work_item_id: work_item_id.clone(),
                next_state: "ready_to_dispatch".to_string(),
                command_id: None,
                idempotency_key: None,
                expected_revision: None,
            },
        )
        .expect("make work item ready");
        (
            state_path,
            crate::project_id(project_root),
            crate::default_workflow_id(project_root),
            work_item_id,
        )
    }

    fn empty_workflow_state_for_db_seed() -> Value {
        json!({
            "workflows": [],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "workflow_node_session_bindings": [],
            "workflow_node_dispatches": [],
            "audit_events": []
        })
    }

    fn db_primary_config(state_path: &Path) -> DbPrimaryJsonProjectionConfig {
        let config_path = storage_mode_path(state_path).expect("storage mode path");
        let runtime_artifacts = config_path.parent().expect("runtime artifacts parent");
        fs::create_dir_all(runtime_artifacts).expect("runtime artifacts parent");
        let canonical_runtime_artifacts =
            fs::canonicalize(runtime_artifacts).expect("canonical runtime artifacts parent");
        DbPrimaryJsonProjectionConfig {
            workflow_state_path: state_path.to_path_buf(),
            confirmed_workflow_state_path: state_path.to_path_buf(),
            db_path: canonical_runtime_artifacts.join("workbench.sqlite"),
            confirmed_db_path: canonical_runtime_artifacts.join("workbench.sqlite"),
            denied_path_markers: vec![],
        }
    }

    fn write_db_primary_config_file(config: &DbPrimaryJsonProjectionConfig) {
        let config_path =
            storage_mode_path(&config.workflow_state_path).expect("storage mode path");
        fs::write(
            config_path,
            serde_json::to_vec_pretty(&json!({
                "schema_version": STORAGE_MODE_SCHEMA_VERSION,
                "mode": DB_PRIMARY_JSON_PROJECTION,
                "workflow_state_path": config.workflow_state_path,
                "confirmed_workflow_state_path": config.confirmed_workflow_state_path,
                "db_path": config.db_path,
                "confirmed_db_path": config.confirmed_db_path,
                "denied_path_markers": config.denied_path_markers,
            }))
            .expect("serialize storage mode config"),
        )
        .expect("write storage mode config")
    }

    fn write_db_primary_config(state_path: &Path) -> DbPrimaryJsonProjectionConfig {
        let config = db_primary_config(state_path);
        write_db_primary_config_file(&config);
        config
    }

    fn seed_db_from_json(config: &DbPrimaryJsonProjectionConfig) {
        let repository = WorkbenchSqliteRepository::open_confirmed(&config.repository_config())
            .expect("initialize confirmed fixture DB");
        let state = crate::read_workflow_state_value(&config.workflow_state_path)
            .expect("read seed workflow state");
        let proposal_store = crate::project_consultation_proposal_store::load_store(
            &config.workflow_state_path,
            crate::unix_timestamp_ms(),
        )
        .expect("read proposal sidecar");
        let authorization_store = crate::plan_authorization_store::load_store(
            &config.workflow_state_path,
            crate::unix_timestamp_ms(),
        )
        .expect("read authorization sidecar");
        let connection = Connection::open(&config.db_path).expect("open fixture DB");
        for proposal in proposal_store.proposals {
            let value = serde_json::to_value(&proposal).expect("proposal value");
            let record_json = serde_json::to_string(&value).expect("proposal json");
            connection
                .execute(
                    "INSERT INTO project_proposals (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        proposal.proposal_id,
                        proposal.project_id,
                        proposal.workflow_id,
                        "m5a-production-seed",
                        crate::workbench_sqlite_importer::canonical_json_hash(&value),
                        record_json,
                    ],
                )
                .expect("seed proposal");
        }
        for audit in proposal_store.audit_events {
            let value = serde_json::to_value(&audit).expect("proposal audit value");
            let record_json = serde_json::to_string(&value).expect("proposal audit json");
            connection
                .execute(
                    "INSERT INTO project_proposal_audit_events (audit_event_id, proposal_id, source_id, record_hash, record_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        audit.audit_event_id,
                        audit.proposal_id,
                        "m5a-production-seed",
                        crate::workbench_sqlite_importer::canonical_json_hash(&value),
                        record_json,
                    ],
                )
                .expect("seed proposal audit");
        }
        for authorization in authorization_store.authorizations {
            let value = serde_json::to_value(&authorization).expect("authorization value");
            let record_json = serde_json::to_string(&value).expect("authorization json");
            connection
                .execute(
                    "INSERT INTO plan_authorizations (authorization_id, source_proposal_id, source_id, record_hash, record_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        authorization.authorization_id,
                        authorization.source_proposal_id,
                        "m5a-production-seed",
                        crate::workbench_sqlite_importer::canonical_json_hash(&value),
                        record_json,
                    ],
                )
                .expect("seed authorization");
        }
        for audit in authorization_store.audit_events {
            let value = serde_json::to_value(&audit).expect("authorization audit value");
            let record_json = serde_json::to_string(&value).expect("authorization audit json");
            connection
                .execute(
                    "INSERT INTO plan_authorization_audit_events (audit_event_id, authorization_id, source_id, record_hash, record_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        audit.audit_event_id,
                        audit.authorization_id,
                        "m5a-production-seed",
                        crate::workbench_sqlite_importer::canonical_json_hash(&value),
                        record_json,
                    ],
                )
                .expect("seed authorization audit");
        }
        let (supervisor_orchestrator_sessions, supervisor_orchestrator_audits) =
            crate::mcp::supervisor_orchestrator::db_primary_projection_records(
                &config.workflow_state_path,
            )
            .expect("read supervisor orchestrator sidecar");
        for session in &supervisor_orchestrator_sessions {
            repository
                .record_supervisor_orchestrator_delta(Some(session), &[], None)
                .expect("seed supervisor orchestrator session");
        }
        for audit in &supervisor_orchestrator_audits {
            repository
                .record_supervisor_orchestrator_delta(None, std::slice::from_ref(audit), None)
                .expect("seed supervisor orchestrator audit");
        }
        m5c::seed_db_from_json(&repository, &config.workflow_state_path)
            .expect("seed M5-C sidecar projection face");
        repository
            .record_workflow_state_delta_with_audit(
                &empty_workflow_state_for_db_seed(),
                &state,
                None,
            )
            .expect("seed complete workflow-state projection face");
        // This test fixture uses the established DB-primary projection writer,
        // whose rows carry its named provenance.  Install the corresponding
        // sidecar-meta row explicitly so the M2 v1 port tests the same exact
        // binding that a real imported sidecar must provide.  The production
        // command still rejects a missing or mismatched binding.
        let state_bytes = fs::read(&config.workflow_state_path)
            .expect("read seeded workflow state for M2 fixture provenance");
        let workspace_id = format!(
            "m2-fixture:{}",
            sha256_hex(&config.workflow_state_path.display().to_string())
        );
        let source_root_hash = sha256_hex_bytes(&state_bytes);
        let meta_json = serde_json::to_string(&json!({
            "schema_version": state.get("schema_version").cloned().unwrap_or(Value::Null),
            "workflow_version": state.get("workflow_version").cloned().unwrap_or(Value::Null),
            "revision": state.get("revision").cloned().unwrap_or(Value::Null),
            "fixture_provenance": "db_primary_projection_writer"
        }))
        .expect("serialize M2 fixture sidecar meta");
        connection
            .execute(
                "INSERT INTO workflow_state_meta
                 (workspace_id, source_root_hash, schema_version, workflow_version, revision, source_id, meta_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    workspace_id,
                    source_root_hash,
                    state.get("schema_version").and_then(Value::as_str).unwrap_or("workflow_state_v0"),
                    state.get("workflow_version").and_then(Value::as_i64).unwrap_or(1),
                    state.get("revision").and_then(Value::as_i64).unwrap_or(0),
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID,
                    meta_json,
                ],
            )
            .expect("seed M2 fixture sidecar-meta provenance");
    }

    fn append_workflow_state_row(state: &mut Value, array_name: &str, row: Value) {
        if state.get(array_name).is_none() {
            state[array_name] = Value::Array(vec![]);
        }
        state
            .get_mut(array_name)
            .and_then(Value::as_array_mut)
            .expect("workflow-state projection array")
            .push(row);
    }

    fn projection_row(key_field: &str, key: &str) -> Value {
        let mut row = serde_json::Map::new();
        row.insert(key_field.to_string(), Value::String(key.to_string()));
        Value::Object(row)
    }

    fn db_primary_fixture(label: &str) -> DbPrimaryFixture {
        clear_storage_mode_cache_for_tests();
        let root = fresh_root(label);
        let project_root = root.join("fixture-project").display().to_string();
        let (state_path, project_id, workflow_id, work_item_id) =
            bootstrap_json_state(&root, &project_root);
        let config = write_db_primary_config(&state_path);
        seed_db_from_json(&config);
        clear_storage_mode_cache_for_path_for_tests(&state_path);
        initialize_for_startup(&state_path).expect("DB primary startup reconciliation");
        DbPrimaryFixture {
            root,
            state_path,
            project_root,
            project_id,
            workflow_id,
            work_item_id,
            config,
        }
    }

    fn proposal_input(fixture: &DbPrimaryFixture) -> crate::CreateProjectConsultationProposalInput {
        crate::CreateProjectConsultationProposalInput {
            project_root: fixture.project_root.clone(),
            project_id: Some(fixture.project_id.clone()),
            workflow_id: Some(fixture.workflow_id.clone()),
            title: "M5-A DB proposal".to_string(),
            user_goal: "verify DB primary projection".to_string(),
            user_requirement_snapshot: "verify DB primary projection".to_string(),
            goal_summary: "exercise proposal DB primary writer".to_string(),
            proposed_steps: vec!["persist proposal".to_string()],
            scope_draft: crate::ProjectConsultationProposalScopeDraft {
                allowed_role_ids: vec!["codex-dev".to_string()],
                allowed_agent_ids: vec![],
                allowed_read_roots: vec![fixture.project_root.clone()],
                allowed_write_roots: vec![fixture.project_root.clone()],
                allowed_tools: vec!["read_file".to_string()],
                allowed_checks: vec![],
                allowed_task_package_kinds: vec!["task_package".to_string()],
                stop_conditions: vec!["user_confirmation".to_string()],
                max_worker_dispatches: Some(1),
                max_runtime_minutes: Some(10),
            },
            risks: vec![crate::ProjectConsultationProposalRisk {
                risk_id: "risk:m5a".to_string(),
                severity: "low".to_string(),
                summary: "fixture only".to_string(),
                mitigation: "temp root".to_string(),
            }],
            worker_acceptance_criteria: vec!["worker evidence".to_string()],
            control_core_acceptance_criteria: vec!["DB row then JSON projection".to_string()],
            supervisor_acceptance_criteria: vec!["supervisor sees projection".to_string()],
            acceptance_criteria: vec!["fixture accepted".to_string()],
            created_by_role: crate::ProjectConsultationProposalCreatorRole::ProjectConsultant,
            suggest_workflow: false,
            tasks: vec![],
            actor_id: "m5a-test".to_string(),
            expected_store_revision: None,
        }
    }

    fn authorization_input(
        fixture: &DbPrimaryFixture,
        proposal_id: &str,
    ) -> crate::CreatePlanAuthorizationInput {
        crate::CreatePlanAuthorizationInput {
            project_root: fixture.project_root.clone(),
            project_id: Some(fixture.project_id.clone()),
            workflow_id: Some(fixture.workflow_id.clone()),
            source_proposal_id: Some(proposal_id.to_string()),
            title: "M5-A DB authorization".to_string(),
            goal_summary: "exercise authorization DB primary writer".to_string(),
            scope: crate::AuthorizedExecutionScope {
                project_id: fixture.project_id.clone(),
                workflow_id: fixture.workflow_id.clone(),
                allowed_role_ids: vec!["codex-dev".to_string()],
                allowed_agent_ids: vec![],
                allowed_read_roots: vec![fixture.project_root.clone()],
                allowed_write_roots: vec![fixture.project_root.clone()],
                allowed_tools: vec!["read_file".to_string()],
                allowed_checks: vec![],
                allowed_task_package_kinds: vec!["task_package".to_string()],
                max_worker_dispatches: Some(1),
                max_runtime_minutes: Some(10),
                stop_conditions: vec![crate::PlanAuthorizationStopCondition {
                    condition_id: "stop:m5a".to_string(),
                    kind: "user_confirmation".to_string(),
                    summary: "fixture stop".to_string(),
                    requires_user_confirmation: true,
                }],
            },
            actor_id: "m5a-test".to_string(),
            actor_role: "project_director".to_string(),
            expires_at_ms: None,
            expected_store_revision: None,
        }
    }

    fn prepare_active_supervisor_run(
        fixture: &DbPrimaryFixture,
    ) -> crate::supervisor_action_controller::SupervisorActionRuntime {
        let proposal = crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(fixture),
            1_700_000_000_400,
            "m5a-blocked-supervisor-setup-proposal",
        )
        .expect("create supervisor setup proposal");
        let authorization = crate::plan_authorization_store::create_authorization(
            &fixture.state_path,
            &authorization_input(fixture, &proposal.proposal.proposal_id),
            1_700_000_000_401,
            "m5a-blocked-supervisor-setup-authorization",
        )
        .expect("create supervisor setup authorization");
        let authorization_id = authorization.authorization.authorization_id;
        crate::plan_authorization_store::record_user_confirmation(
            &fixture.state_path,
            &crate::RecordPlanAuthorizationUserConfirmationInput {
                project_root: fixture.project_root.clone(),
                authorization_id: authorization_id.clone(),
                actor_id: "m5a-test".to_string(),
                confirmation_summary: "M5-A blocked fallback fixture confirmation".to_string(),
                expected_store_revision: None,
            },
            1_700_000_000_402,
            "m5a-blocked-supervisor-setup-user-confirmation",
        )
        .expect("confirm supervisor setup authorization");
        crate::plan_authorization_store::record_global_boundary_review(
            &fixture.state_path,
            &crate::RecordPlanAuthorizationGlobalBoundaryReviewInput {
                project_root: fixture.project_root.clone(),
                authorization_id: authorization_id.clone(),
                actor_id: "m5a-test".to_string(),
                review_status: "approved".to_string(),
                summary: "M5-A blocked fallback fixture review".to_string(),
                source_proposal_id: Some(proposal.proposal.proposal_id),
                checklist: None,
                findings: vec![],
                reviewed_scope_fingerprint: None,
                expected_store_revision: None,
            },
            1_700_000_000_403,
            "m5a-blocked-supervisor-setup-boundary-review",
        )
        .expect("activate supervisor setup authorization");

        let supervisor_node_id = format!("{}:node:codex-dev", fixture.workflow_id);
        let mut state = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read supervisor setup workflow state");
        state["workflow_node_dispatches"]
            .as_array_mut()
            .expect("workflow dispatch array")
            .push(json!({
                "state": "prepared",
                "prompt_kind": "authorized_prepared_auto_dispatch",
                "project_id": fixture.project_id,
                "workflow_id": fixture.workflow_id,
                "plan_authorization_id": authorization_id,
                "node_id": supervisor_node_id,
                "work_item_id": fixture.work_item_id,
            }));
        crate::write_validated_workflow_state(&fixture.state_path, &state)
            .expect("write supervisor setup prepared dispatch");

        let runtime = crate::supervisor_action_controller::SupervisorActionRuntime {
            run_id: "supervisor:m5a:blocked-json-fallback".to_string(),
            project_root: fixture.project_root.clone(),
            workflow_id: fixture.workflow_id.clone(),
            authorization_id,
            workflow_state_path: fixture.state_path.clone(),
            quota_limits: crate::mcp::SupervisorQuotaLimits {
                max_active_workers: 1,
                max_follow_ups_per_worker: 0,
                max_runtime_minutes: 1,
            },
            started_at_ms: crate::unix_timestamp_ms(),
        };
        let config = crate::mcp::McpServerConfig {
            role: crate::mcp::McpRole::SupervisorOrchestrator,
            run_id: runtime.run_id.clone(),
            node_id: None,
            supervisor_workflow_state_path: Some(runtime.workflow_state_path.clone()),
            supervisor_quota_limits: Some(runtime.quota_limits),
            knowledge_open_relay: None,
        };
        crate::mcp::supervisor_orchestrator::record_pilot_session_started(
            &config,
            &crate::mcp::supervisor_orchestrator::SupervisorPilotSessionLaunch {
                project_root: runtime.project_root.clone(),
                workflow_id: runtime.workflow_id.clone(),
                authorization_id: runtime.authorization_id.clone(),
                model_id: "m5a-test".to_string(),
                reasoning_effort: "medium".to_string(),
                workbench_executable_path: "/tmp/m5a-test-workbench".to_string(),
                workbench_build_id: "m5a-test-build".to_string(),
                supervisor_contract_version: "supervisor_action_proposal.v1".to_string(),
                supervisor_contract_sha256: "m5a-test-supervisor-contract".to_string(),
                worker_report_contract_sha256: "m5a-test-worker-report-contract".to_string(),
            },
        )
        .expect("record supervisor setup run");
        runtime
    }

    fn dispatch_context(fixture: &DbPrimaryFixture) -> crate::WorkflowNodeDispatchContext {
        crate::WorkflowNodeDispatchContext {
            project_id: fixture.project_id.clone(),
            workflow_id: fixture.workflow_id.clone(),
            node_id: format!("{}:node:codex-dev", fixture.workflow_id),
            work_item_id: fixture.work_item_id.clone(),
            work_item_state: "ready_to_dispatch".to_string(),
            binding_id: "binding:m5a".to_string(),
            native_thread_id: "thread:m5a".to_string(),
            prompt_preview: "M5-A DB primary fixture".to_string(),
            prompt_kind: "safe_probe".to_string(),
            memory_packet_snapshot_id: None,
            memory_packet_fingerprint: None,
            plan_authorization_id: None,
            authorization_check: None,
            prepared_dispatch_id: None,
            user_reviewed_instruction: None,
            warnings: vec![],
        }
    }

    fn db_primary_row_counts(config: &DbPrimaryJsonProjectionConfig) -> Vec<(&'static str, i64)> {
        let connection =
            Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open fixture DB read only");
        [
            "project_proposals",
            "project_proposal_audit_events",
            "plan_authorizations",
            "plan_authorization_audit_events",
            "workflow_node_dispatches",
            "work_items",
            "workflow_nodes",
            "supervisor_actions",
            "supervisor_orchestrator_sessions",
            "supervisor_orchestrator_audit_events",
            "workflow_audit_events",
        ]
        .into_iter()
        .map(|table| {
            let count = connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count fixture DB table");
            (table, count)
        })
        .collect()
    }

    // Counts only the M2 transaction foundation tables exercised by the
    // workflow-state reference slice.  This deliberately does not turn the
    // storage-mode fixture into a claim about unrelated product flows.
    fn m2_workflow_state_ledger_counts(config: &DbPrimaryJsonProjectionConfig) -> [i64; 5] {
        let connection =
            Connection::open_with_flags(&config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open M2 fixture DB read only");
        [
            "command_receipts",
            "events",
            "audit_records",
            "outbox_items",
            "current_snapshots",
        ]
        .map(|table| {
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count M2 reference-slice table")
        })
    }

    fn degradation_audits(state_path: &Path) -> Vec<Value> {
        crate::read_workflow_state_value(state_path)
            .expect("read workflow state for degradation audit")
            .get("audit_events")
            .and_then(Value::as_array)
            .expect("workflow audit array")
            .iter()
            .filter(|event| {
                event.get("event_type").and_then(Value::as_str)
                    == Some("storage_mode_degraded_json_only")
            })
            .cloned()
            .collect()
    }

    fn assert_db_primary_health_blocked(state_path: &Path, expected_reason: &str) {
        let health = health_cache().lock().expect("storage mode health lock");
        let Some(DbPrimaryHealth::Blocked(reason)) = health.get(state_path) else {
            panic!("DB primary health must remain blocked");
        };
        assert!(reason.contains(expected_reason), "{reason}");
    }

    #[test]
    fn m5a_missing_malformed_or_mismatched_config_is_json_only() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        clear_storage_mode_cache_for_tests();
        let root = fresh_root("config-fail-closed");
        let state_path = root.join("workflow-state").join("workflow-state.v0.json");
        fs::create_dir_all(state_path.parent().expect("state parent")).expect("state parent");
        fs::write(&state_path, "{}").expect("state placeholder");
        assert!(matches!(
            storage_mode_for(&state_path),
            StorageMode::JsonOnly { .. }
        ));

        let config_path = storage_mode_path(&state_path).expect("config path");
        fs::create_dir_all(config_path.parent().expect("config parent")).expect("config parent");
        fs::write(&config_path, "not json").expect("malformed config");
        clear_storage_mode_cache_for_tests();
        assert!(matches!(
            storage_mode_for(&state_path),
            StorageMode::JsonOnly { .. }
        ));

        let runtime_artifacts = fs::canonicalize(config_path.parent().expect("config parent"))
            .expect("canonical config parent");
        let db_path = runtime_artifacts.join("workbench.sqlite");
        fs::write(
            &config_path,
            serde_json::to_vec(&json!({
                "schema_version": STORAGE_MODE_SCHEMA_VERSION,
                "mode": DB_PRIMARY_JSON_PROJECTION,
                "workflow_state_path": root.join("wrong-workflow-state.json"),
                "confirmed_workflow_state_path": root.join("wrong-workflow-state.json"),
                "db_path": db_path,
                "confirmed_db_path": db_path,
            }))
            .expect("mismatched config json"),
        )
        .expect("mismatched config");
        clear_storage_mode_cache_for_tests();
        assert!(matches!(
            storage_mode_for(&state_path),
            StorageMode::JsonOnly { .. }
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m5a_explicit_json_only_keeps_proposal_bytes_identical_to_missing_config() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        clear_storage_mode_cache_for_tests();
        let shared_project_root = "/tmp/m5a-json-only-byte-equivalence";
        let first_root = fresh_root("json-only-missing");
        let second_root = fresh_root("json-only-explicit");
        let (first_state, first_project_id, first_workflow_id, _) =
            bootstrap_json_state(&first_root, shared_project_root);
        let (second_state, second_project_id, second_workflow_id, _) =
            bootstrap_json_state(&second_root, shared_project_root);
        assert_eq!(first_project_id, second_project_id);
        assert_eq!(first_workflow_id, second_workflow_id);
        let explicit_config = storage_mode_path(&second_state).expect("explicit config path");
        fs::create_dir_all(explicit_config.parent().expect("explicit config parent"))
            .expect("explicit config parent");
        fs::write(
            &explicit_config,
            serde_json::to_vec(&json!({
                "schema_version": STORAGE_MODE_SCHEMA_VERSION,
                "mode": JSON_ONLY,
            }))
            .expect("explicit json-only config"),
        )
        .expect("write explicit json-only config");
        let input = crate::CreateProjectConsultationProposalInput {
            project_root: shared_project_root.to_string(),
            project_id: Some(first_project_id),
            workflow_id: Some(first_workflow_id),
            title: "byte-equivalent proposal".to_string(),
            user_goal: "verify json-only bytes".to_string(),
            user_requirement_snapshot: "verify json-only bytes".to_string(),
            goal_summary: "same fixed mutation".to_string(),
            proposed_steps: vec!["write proposal".to_string()],
            scope_draft: crate::ProjectConsultationProposalScopeDraft {
                allowed_role_ids: vec!["codex-dev".to_string()],
                allowed_agent_ids: vec![],
                allowed_read_roots: vec![shared_project_root.to_string()],
                allowed_write_roots: vec![shared_project_root.to_string()],
                allowed_tools: vec![],
                allowed_checks: vec![],
                allowed_task_package_kinds: vec!["task_package".to_string()],
                stop_conditions: vec![],
                max_worker_dispatches: Some(1),
                max_runtime_minutes: Some(1),
            },
            risks: vec![],
            worker_acceptance_criteria: vec!["worker".to_string()],
            control_core_acceptance_criteria: vec!["core".to_string()],
            supervisor_acceptance_criteria: vec!["supervisor".to_string()],
            acceptance_criteria: vec!["legacy".to_string()],
            created_by_role: crate::ProjectConsultationProposalCreatorRole::ProjectConsultant,
            suggest_workflow: false,
            tasks: vec![],
            actor_id: "m5a-byte-test".to_string(),
            expected_store_revision: None,
        };
        crate::project_consultation_proposal_store::create_proposal(
            &first_state,
            &input,
            1_700_000_000_000,
            "m5a-byte-write",
        )
        .expect("missing-config proposal");
        clear_storage_mode_cache_for_tests();
        crate::project_consultation_proposal_store::create_proposal(
            &second_state,
            &input,
            1_700_000_000_000,
            "m5a-byte-write",
        )
        .expect("explicit-json-only proposal");
        let first_bytes = fs::read(
            crate::project_consultation_proposal_store::sidecar_path(&first_state)
                .expect("first proposal sidecar"),
        )
        .expect("read first sidecar");
        let second_bytes = fs::read(
            crate::project_consultation_proposal_store::sidecar_path(&second_state)
                .expect("second proposal sidecar"),
        )
        .expect("read second sidecar");
        assert_eq!(first_bytes, second_bytes);
        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
    }

    #[test]
    fn m5a_production_seed_shape_reconciles_legacy_hashes_and_audit_tables() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        clear_storage_mode_cache_for_tests();
        let root = fresh_root("production-seed-shape");
        let project_root = root.join("fixture-project").display().to_string();
        let (state_path, project_id, workflow_id, work_item_id) =
            bootstrap_json_state(&root, &project_root);
        let config = db_primary_config(&state_path);
        let fixture = DbPrimaryFixture {
            root,
            state_path,
            project_root,
            project_id,
            workflow_id,
            work_item_id,
            config,
        };

        let proposal = crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(&fixture),
            1_700_000_000_050,
            "m5a-production-seed-proposal",
        )
        .expect("JSON-only proposal before storage mode exists");
        crate::plan_authorization_store::create_authorization(
            &fixture.state_path,
            &authorization_input(&fixture, &proposal.proposal.proposal_id),
            1_700_000_000_051,
            "m5a-production-seed-authorization",
        )
        .expect("JSON-only authorization before storage mode exists");

        write_db_primary_config_file(&fixture.config);
        seed_db_from_json(&fixture.config);
        clear_storage_mode_cache_for_tests();
        initialize_for_startup(&fixture.state_path)
            .expect("production-shaped DB seed must reconcile before DB-primary writes");
        let report =
            reconcile_db_vs_json(&fixture.config).expect("reconcile production-shaped seed");
        assert!(
            report.is_green(),
            "production-shaped seed must be green: {report:?}"
        );
        let audit_table = report
            .tables
            .iter()
            .find(|table| table.table_name == "workflow_audit_events")
            .expect("audit reconciliation table");
        assert_eq!(audit_table.db_count, audit_table.json_count);
        assert_eq!(audit_table.matched_count, audit_table.db_count);
    }

    #[test]
    fn m5b_full_workflow_projection_face_is_seeded_and_reconciles() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m5b-full-workflow-projection-face");
        let report = reconcile_db_vs_json(&fixture.config).expect("reconcile full projection face");
        let table_names = report
            .tables
            .iter()
            .map(|table| table.table_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            table_names,
            vec![
                "project_proposals",
                "plan_authorizations",
                "projects",
                "agent_adapters",
                "workflows",
                "workflow_nodes",
                "workflow_edges",
                "work_items",
                "workflow_artifacts",
                "workflow_reviews",
                "workflow_node_session_bindings",
                "workflow_node_dispatches",
                "execution_attempts",
                "workflow_chain_runs",
                "workflow_execution_controls",
                "permission_requests",
                "capabilities",
                "harness_resources",
                "supervisor_actions",
                "supervisor_orchestrator_sessions",
                "supervisor_orchestrator_audit_events",
                "workflow_audit_events",
                "supervisor_reviews",
                "supervisor_review_audit_events",
                "supervisor_boundary_reviews",
                "supervisor_boundary_audit_events",
                "session_continuations",
                "session_continuation_attempts",
                "session_continuation_audit_events",
                "runtime_log_entries",
                "runtime_log_summaries",
                "product_commands",
                "product_command_previews",
                "product_command_decisions",
                "product_command_attempts",
            ]
        );
        for table in &report.tables {
            assert_eq!(table.db_count, table.json_count, "{table:?}");
            assert_eq!(table.matched_count, table.db_count, "{table:?}");
            assert!(table.db_leading.is_empty(), "{table:?}");
            assert!(table.json_leading.is_empty(), "{table:?}");
            assert!(table.hash_mismatches.is_empty(), "{table:?}");
        }
        let projects = report
            .tables
            .iter()
            .find(|table| table.table_name == "projects")
            .expect("project reconciliation table");
        assert_eq!(projects.db_count, 1, "{projects:?}");
        assert_eq!(projects.json_count, 1, "{projects:?}");
        let agent_adapters = report
            .tables
            .iter()
            .find(|table| table.table_name == "agent_adapters")
            .expect("agent adapter reconciliation table");
        assert_eq!(agent_adapters.db_count, 1, "{agent_adapters:?}");
        assert_eq!(agent_adapters.json_count, 1, "{agent_adapters:?}");
    }
    include!("workbench_sqlite_storage_mode_m5b_tests.rs");
    include!("workbench_sqlite_storage_mode_m5c_tests.rs");

    #[test]
    fn m5b_batch2_bridge_commits_workflow_audit_before_json_projection() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m5b-batch2-bridge");
        let mut next = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read DB-primary workflow state");
        append_workflow_state_row(
            &mut next,
            "audit_events",
            json!({
                "event_id": "audit:m5b:batch2-bridge",
                "event_type": "m5b_batch2_bridge_test",
                "target_ref": "m5b-batch2-bridge",
                "actor_ref": "m5b-test",
                "source_kind": "workspace_state",
                "permission_level": "user_confirmed_write",
                "created_at": "1700000000600",
                "reason": "exercise explicit Batch 2 DB-primary bridge"
            }),
        );
        crate::write_m5b_batch2_workflow_state(
            &fixture.state_path,
            "m5b_batch2_bridge_test",
            &next,
        )
        .expect("Batch 2 bridge writes DB then JSON projection");
        let report = reconcile_db_vs_json(&fixture.config).expect("reconcile Batch 2 bridge");
        assert!(report.is_green(), "{report:?}");
        let audit_table = report
            .tables
            .iter()
            .find(|table| table.table_name == "workflow_audit_events")
            .expect("workflow audit table");
        assert!(
            audit_table.db_count.gt(&0),
            "Batch 2 bridge must write a workflow audit row"
        );
    }

    #[test]
    fn m5b_batch2_bootstraps_second_project_through_db_primary_and_reconciles() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m5b-batch2-bootstrap");
        // Keep the differing project segment inside stable_id's historic 96-character identity.
        let second_project_root = "/tmp/m5b-batch2-second-project";
        crate::bootstrap_project_workflow_at(
            &fixture.state_path,
            &project_record(second_project_root),
        )
        .expect("DB-primary bootstrap for a second project");

        let report = reconcile_db_vs_json(&fixture.config)
            .expect("reconcile DB-primary second-project bootstrap");
        assert!(report.is_green(), "{report:?}");
        let workflows = report
            .tables
            .iter()
            .find(|table| table.table_name == "workflows")
            .expect("workflow reconciliation table");
        assert_eq!(workflows.db_count, 2, "{workflows:?}");
        assert_eq!(workflows.json_count, 2, "{workflows:?}");
        assert_eq!(workflows.matched_count, 2, "{workflows:?}");
        let projects = report
            .tables
            .iter()
            .find(|table| table.table_name == "projects")
            .expect("project reconciliation table");
        assert_eq!(projects.db_count, 2, "{projects:?}");
        assert_eq!(projects.json_count, 2, "{projects:?}");
        assert_eq!(projects.matched_count, 2, "{projects:?}");
    }

    #[test]
    fn m5b_db_ahead_replays_full_workflow_projection_face() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m5b-full-workflow-replay");
        let before = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read sparse workflow state");
        let mut after = before.clone();
        let rows = [
            ("projects", "project_id", "project:m5b:db-ahead"),
            ("agent_adapters", "adapter_id", "adapter:m5b:db-ahead"),
            ("workflows", "workflow_id", "workflow:m5b:db-ahead"),
            ("nodes", "node_id", "node:m5b:db-ahead"),
            ("edges", "edge_id", "edge:m5b:db-ahead"),
            ("work_items", "work_item_id", "work-item:m5b:db-ahead"),
            ("artifacts", "artifact_id", "artifact:m5b:db-ahead"),
            ("reviews", "review_id", "review:m5b:db-ahead"),
            (
                "workflow_node_session_bindings",
                "binding_id",
                "binding:m5b:db-ahead",
            ),
            (
                "workflow_node_dispatches",
                "dispatch_id",
                "dispatch:m5b:db-ahead",
            ),
            ("execution_attempts", "attempt_id", "attempt:m5b:db-ahead"),
            ("workflow_chain_runs", "chain_run_id", "chain:m5b:db-ahead"),
            (
                "workflow_execution_controls",
                "control_id",
                "control:m5b:db-ahead",
            ),
            (
                "permission_requests",
                "request_id",
                "permission:m5b:db-ahead",
            ),
            ("capabilities", "capability_id", "capability:m5b:db-ahead"),
            ("harness_resources", "resource_id", "harness:m5b:db-ahead"),
        ];
        for (array_name, key_field, key) in rows {
            append_workflow_state_row(&mut after, array_name, projection_row(key_field, key));
        }
        append_workflow_state_row(
            &mut after,
            "audit_events",
            json!({
                "event_id": "audit:m5b:db-ahead",
                "event_type": "m5b_db_ahead_replay",
                "target_ref": "m5b-full-workflow-replay",
                "actor_ref": "m5b-test",
                "source_kind": "workspace_state",
                "permission_level": "user_confirmed_write",
                "created_at": "1700000000601",
                "reason": "simulate DB commit before JSON projection"
            }),
        );
        primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository")
            .record_workflow_state_delta_with_audit(&before, &after, None)
            .expect("commit full DB-leading workflow projection face");

        clear_storage_mode_cache_for_tests();
        initialize_for_startup(&fixture.state_path)
            .expect("restart replays every DB-leading workflow table");
        let replayed = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read replayed workflow state");
        for (array_name, key_field, key) in rows {
            assert!(
                replayed[array_name]
                    .as_array()
                    .is_some_and(|records| records.iter().any(|record| {
                        record.get(key_field).and_then(Value::as_str) == Some(key)
                    })),
                "replay missing {array_name}:{key}"
            );
        }
        assert!(
            replayed["audit_events"]
                .as_array()
                .is_some_and(|records| records
                    .iter()
                    .any(|record| record["event_id"] == "audit:m5b:db-ahead")),
            "replay missing DB-leading workflow audit"
        );
        assert!(reconcile_db_vs_json(&fixture.config)
            .expect("reconcile replayed full workflow projection face")
            .is_green());
    }

    #[test]
    fn m5b_unordered_edge_and_artifact_divergence_fail_closed_as_hash_mismatch() {
        for (table_name, key_field) in [
            ("workflow_edges", "edge_id"),
            ("workflow_artifacts", "artifact_id"),
        ] {
            let key = format!("{table_name}:m5b:unordered");
            let db_value = projection_row(key_field, &key);
            let mut json_value = projection_row(key_field, &key);
            json_value["different"] = Value::Bool(true);
            let table = reconcile_table(
                table_name,
                vec![DbRecord {
                    natural_key: key.clone(),
                    record_hash: record_hash(&db_value).expect("DB record hash"),
                    value: db_value,
                }],
                vec![DbRecord {
                    natural_key: key.clone(),
                    record_hash: record_hash(&json_value).expect("JSON record hash"),
                    value: json_value,
                }],
            );
            assert_eq!(table.hash_mismatches, vec![key], "{table:?}");
            assert!(table.db_leading.is_empty(), "{table:?}");
            assert!(table.json_leading.is_empty(), "{table:?}");
        }
    }

    #[test]
    fn m2_versioned_db_record_is_db_leading_over_an_unversioned_legacy_projection() {
        let key = "workflow:m2:node:codex-dev".to_string();
        let db_value = json!({
            "node_id": key,
            "state": "running",
            "workflow_revision_after": 7
        });
        let json_value = json!({
            "node_id": key,
            "state": "draft"
        });
        let table = reconcile_table(
            "workflow_nodes",
            vec![DbRecord {
                natural_key: key.clone(),
                record_hash: record_hash(&db_value).expect("DB record hash"),
                value: db_value,
            }],
            vec![DbRecord {
                natural_key: key.clone(),
                record_hash: record_hash(&json_value).expect("JSON record hash"),
                value: json_value,
            }],
        );

        assert_eq!(table.db_leading, vec![key], "{table:?}");
        assert!(table.json_leading.is_empty(), "{table:?}");
        assert!(table.hash_mismatches.is_empty(), "{table:?}");
    }

    #[test]
    fn m5a_db_primary_writes_five_product_flows_and_reconciles_without_lag() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("five-flows");
        let proposal = crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(&fixture),
            1_700_000_000_100,
            "m5a-proposal",
        )
        .expect("DB-primary proposal");
        let authorization = crate::plan_authorization_store::create_authorization(
            &fixture.state_path,
            &authorization_input(&fixture, &proposal.proposal.proposal_id),
            1_700_000_000_101,
            "m5a-authorization",
        )
        .expect("DB-primary authorization");
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository");
        let mut stale_authorization =
            serde_json::to_value(&authorization.authorization).expect("authorization JSON");
        let stale = stale_authorization
            .as_object_mut()
            .expect("authorization object");
        stale.insert(
            "proposal_id".to_string(),
            Value::String(proposal.proposal.proposal_id.clone()),
        );
        stale.insert("revision".to_string(), Value::from(1_i64));
        let cas_error = repository
            .save_authorization_with_audit(
                &stale_authorization,
                0,
                &RepositoryAuditEntry {
                    event_id: "audit:m5a:authorization-cas-conflict".to_string(),
                    target_kind: "plan_authorization".to_string(),
                    target_id: authorization.authorization.authorization_id.clone(),
                    payload: json!({"event_id":"audit:m5a:authorization-cas-conflict"}),
                },
                None,
            )
            .expect_err("stale authorization revision must fail");
        assert!(
            cas_error.contains("authorization_cas_conflict"),
            "{cas_error}"
        );

        let context = dispatch_context(&fixture);
        crate::write_prepared_dispatch(&fixture.state_path, context.clone())
            .expect("DB-primary prepared dispatch");
        crate::write_started_dispatch(&fixture.state_path, &context)
            .expect("DB-primary started dispatch");
        crate::update_work_item_state_at(
            &fixture.state_path,
            &crate::WorkItemStateUpdateRequest {
                project_root: fixture.project_root.clone(),
                work_item_id: fixture.work_item_id.clone(),
                next_state: "ready_for_review".to_string(),
                command_id: None,
                idempotency_key: None,
                expected_revision: None,
            },
        )
        .expect("DB-primary work item transition");

        let report = reconcile_db_vs_json(&fixture.config).expect("reconcile DB and JSON");
        assert!(
            report.is_green(),
            "reconciliation must be green: {report:?}"
        );
        let dispatch_table = report
            .tables
            .iter()
            .find(|table| table.table_name == "workflow_node_dispatches")
            .expect("dispatch reconciliation table");
        assert_eq!(dispatch_table.db_count, 2);
        assert_eq!(dispatch_table.matched_count, 2);
        assert!(report
            .tables
            .iter()
            .all(|table| table.db_leading.is_empty()));
        assert!(report
            .tables
            .iter()
            .all(|table| table.json_leading.is_empty()));
        assert!(report
            .tables
            .iter()
            .all(|table| table.hash_mismatches.is_empty()));
    }

    #[test]
    fn m2_green_db_primary_restart_does_not_rewrite_startup_audit_or_projection() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-green-restart-is-read-only");
        let json_before = fs::read(&fixture.state_path).expect("read green JSON projection");
        let db_before = fs::read(&fixture.config.db_path).expect("read green DB projection");
        let audit_count_before = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read green workflow state")
            .get("audit_events")
            .and_then(Value::as_array)
            .map(Vec::len)
            .expect("workflow audit array");

        // The cache clear models a new App process.  A green restart may
        // reconcile and verify, but must not manufacture another startup
        // audit or alter either primary/projection artifact.
        clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
        initialize_for_startup(&fixture.state_path).expect("green restart reconciliation");

        assert_eq!(
            fs::read(&fixture.state_path).expect("read JSON after restart"),
            json_before,
            "green restart must not rewrite the JSON projection"
        );
        assert_eq!(
            fs::read(&fixture.config.db_path).expect("read DB after restart"),
            db_before,
            "green restart must not append a duplicate SQLite startup audit"
        );
        let audit_count_after = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read workflow state after restart")
            .get("audit_events")
            .and_then(Value::as_array)
            .map(Vec::len)
            .expect("workflow audit array after restart");
        assert_eq!(audit_count_after, audit_count_before);
    }

    #[test]
    fn m2_reference_slice_db_primary_commits_internal_projection_and_replays_once() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-outbox");
        let before = m2_workflow_state_ledger_counts(&fixture.config);
        assert_eq!(before, [0, 0, 0, 0, 0], "fixture must start ledger-clean");
        let expected_revision = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository")
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read authoritative M2 workflow revision");

        let request = crate::WorkItemStateUpdateRequest {
            project_root: fixture.project_root.clone(),
            work_item_id: fixture.work_item_id.clone(),
            next_state: "running".to_string(),
            command_id: Some("m2-reference-slice-replay-command".to_string()),
            idempotency_key: Some("m2-reference-slice-replay-key".to_string()),
            expected_revision: Some(expected_revision),
        };
        let first = crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect("DB-primary M2 reference-slice transition");
        let receipt_id = first.receipt_id.clone().expect("M2 receipt id");
        let json_after_first = fs::read(&fixture.state_path).expect("read first JSON projection");

        let connection =
            Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open M2 fixture DB read only");
        let command_id = "m2-reference-slice-replay-command".to_string();
        let receipt_status: String = connection
            .query_row(
                "SELECT status FROM command_receipts WHERE receipt_id = ?1 AND command_id = ?2",
                params![receipt_id, command_id],
                |row| row.get(0),
            )
            .expect("load completed M2 receipt");
        assert_eq!(receipt_status, "COMMITTED");
        let outbox_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM outbox_items", [], |row| row.get(0))
            .expect("count external effects");
        assert_eq!(
            outbox_count, 0,
            "the JSON sidecar is internal/rebuildable and must not create an outbox effect"
        );
        let event_id: String = connection
            .query_row(
                "SELECT event_id FROM events WHERE command_id = ?1 AND event_type = 'WorkItemStateUpdated'",
                [command_id.as_str()],
                |row| row.get(0),
            )
            .expect("load the reference-slice domain event");
        let (checkpoint_version, checkpoint_last_event, checkpoint_watermark, checkpoint_status, checkpoint_error):
            (String, Option<String>, String, String, Option<String>) = connection
            .query_row(
                "SELECT projector_version, last_event_id, source_watermark, status, error_receipt_ref
                 FROM projection_checkpoints
                 WHERE projector_id = ?1",
                [crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .expect("record the completed reference-slice checkpoint");
        assert_eq!(
            checkpoint_version,
            crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION
        );
        assert_eq!(checkpoint_last_event.as_deref(), Some(event_id.as_str()));
        assert_eq!(checkpoint_watermark, event_id);
        assert_eq!(checkpoint_status, "CAUGHT_UP");
        assert_eq!(checkpoint_error, None);
        let authoritative_snapshot_hash: String = connection
            .query_row(
                "SELECT snapshot_hash FROM current_snapshots
                 WHERE object_ref = ?1 AND projector_id = 'workflow_projector'",
                [format!("workflow_state:{}:{}", fixture.project_root, fixture.workflow_id)],
                |row| row.get(0),
            )
            .expect("load authoritative snapshot hash");
        assert_eq!(
            authoritative_snapshot_hash.len(),
            64,
            "the authoritative snapshot uses a SHA-256 canonical content hash"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [1, 1, 1, 0, 1],
            "the M2 UoW must not manufacture an external effect"
        );
        let state = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read projected workflow state");
        assert_eq!(state["work_items"][0]["state"], "running");
        assert!(
            state["work_items"][0]["workflow_revision_after"]
                .as_i64()
                .is_some_and(|revision| revision > 0),
            "M2 changes must carry a receipt-backed ordering field for DB-leading recovery"
        );
        assert!(
            state["nodes"]
                .as_array()
                .is_some_and(|nodes| nodes.iter().any(|node| {
                    node["state"] == "running"
                        && node["workflow_revision_after"]
                            .as_i64()
                            .is_some_and(|revision| revision > 0)
                })),
            "the transitioned node must carry the same M2 ordering mechanism"
        );

        let replay = crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect("completed M2 receipt may replay");
        assert_eq!(replay.receipt_id.as_deref(), Some(receipt_id.as_str()));
        assert_eq!(
            fs::read(&fixture.state_path).expect("read replayed JSON projection"),
            json_after_first,
            "completed replay must not rewrite the JSON projection"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [1, 1, 1, 0, 1],
            "completed replay must not grow the M2 ledger"
        );
    }

    #[test]
    fn m2_reference_slice_external_pending_owner_replays_without_duplicate_mutation() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-external-pending-replay");
        let expected_revision = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository")
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read authoritative M2 workflow revision");
        let request = crate::WorkItemStateUpdateRequest {
            project_root: fixture.project_root.clone(),
            work_item_id: fixture.work_item_id.clone(),
            next_state: "running".to_string(),
            command_id: Some("m2-reference-slice-external-pending-command".to_string()),
            idempotency_key: Some("m2-reference-slice-external-pending-key".to_string()),
            expected_revision: Some(expected_revision),
        };
        let first = crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect("initial M2 reference-slice transition");
        let receipt_id = first.receipt_id.expect("first receipt");
        let json_before_replay = fs::read(&fixture.state_path).expect("read first projection");
        let ledger_before_replay = m2_workflow_state_ledger_counts(&fixture.config);
        let connection = Connection::open(&fixture.config.db_path).expect("open fixture DB write");
        let rows = connection
            .execute(
                "UPDATE command_receipts SET status = 'EXTERNAL_PENDING'
                 WHERE receipt_id = ?1 AND command_id = ?2 AND status = 'COMMITTED'",
                params![receipt_id, "m2-reference-slice-external-pending-command"],
            )
            .expect("mark exact existing owner as externally pending");
        assert_eq!(rows, 1, "only the completed owning receipt may transition");
        drop(connection);

        let replay = crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect("an external-pending owner still replays its immutable receipt");
        assert_eq!(replay.receipt_id.as_deref(), Some(receipt_id.as_str()));
        assert_eq!(
            fs::read(&fixture.state_path).expect("read replay projection"),
            json_before_replay,
            "external-pending replay may not rewrite the JSON projection"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            ledger_before_replay,
            "external-pending replay may not create another receipt/event/audit/outbox/snapshot"
        );
    }

    #[test]
    fn m2_reference_slice_checkpoint_rejects_projection_hash_mismatch_without_advancing() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-snapshot-parity");
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository");
        let object_ref = format!("workflow_state:{}:{}", fixture.project_root, fixture.workflow_id);
        let watermark = "event:m2-snapshot-parity";
        let content_hash = crate::m2_update_work_item_state::canonical_workflow_state_sidecar_snapshot_hash(
            &fixture.project_root,
            &fixture.workflow_id,
            11,
            &json!({"work_item_id": fixture.work_item_id, "state": "running"}),
            &json!({"node_id": "node:m2-snapshot-parity", "state": "running"}),
        );
        let mismatched_hash = crate::m2_update_work_item_state::canonical_workflow_state_sidecar_snapshot_hash(
            &fixture.project_root,
            &fixture.workflow_id,
            11,
            &json!({"work_item_id": fixture.work_item_id, "state": "completed"}),
            &json!({"node_id": "node:m2-snapshot-parity", "state": "completed"}),
        );
        repository
            .with_immediate_transaction(
                "m2_snapshot_parity_seed_authoritative",
                None,
                |transaction| crate::workbench_sqlite_repository::record_m2_workflow_state_sidecar_snapshot_in_transaction(
                    transaction,
                    &object_ref,
                    11,
                    watermark,
                    &content_hash,
                    1_800_000_000_000,
                ),
            )
            .expect("persist authoritative canonical snapshot");
        repository
            .with_immediate_transaction(
                "m2_snapshot_parity_checkpoint_match",
                None,
                |transaction| crate::workbench_sqlite_repository::record_m2_workflow_state_sidecar_projection_checkpoint_in_transaction(
                    transaction,
                    &object_ref,
                    11,
                    watermark,
                    "receipt:m2-snapshot-parity",
                    &content_hash,
                    1_800_000_000_001,
                ),
            )
            .expect("matching internal projection advances checkpoint");
        let mismatch = repository
            .with_immediate_transaction(
                "m2_snapshot_parity_checkpoint_mismatch",
                None,
                |transaction| crate::workbench_sqlite_repository::record_m2_workflow_state_sidecar_projection_checkpoint_in_transaction(
                    transaction,
                    &object_ref,
                    11,
                    watermark,
                    "receipt:m2-snapshot-parity-forged",
                    &mismatched_hash,
                    1_800_000_000_002,
                ),
            )
            .expect_err("mismatched projection may not advance a checkpoint");
        assert!(mismatch.contains("m2_workflow_state_projection_snapshot_hash_mismatch"), "{mismatch}");

        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open checkpoint DB read only");
        let (checkpoint_status, checkpoint_watermark, stored_hash): (String, String, String) = connection
            .query_row(
                "SELECT checkpoints.status, checkpoints.source_watermark, snapshots.snapshot_hash
                 FROM projection_checkpoints AS checkpoints
                 JOIN current_snapshots AS snapshots
                   ON snapshots.projector_id = 'workflow_projector'
                 WHERE checkpoints.projector_id = ?1 AND snapshots.object_ref = ?2",
                [
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                    object_ref.as_str(),
                ],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read retained authoritative snapshot and checkpoint");
        assert_eq!(checkpoint_status, "CAUGHT_UP");
        assert_eq!(checkpoint_watermark, watermark, "mismatch may not forge a later checkpoint");
        assert_eq!(stored_hash, content_hash, "source snapshot remains durable for recovery");
    }

    #[test]
    fn m2_reference_slice_full_aggregate_snapshot_rebuilds_checkpoint_after_crash_window() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-full-aggregate-checkpoint-recovery");
        let expected_revision = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository")
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read authoritative workflow revision");
        let request = crate::WorkItemStateUpdateRequest {
            project_root: fixture.project_root.clone(),
            work_item_id: fixture.work_item_id.clone(),
            next_state: "running".to_string(),
            command_id: Some("m2-reference-slice-full-aggregate-command".to_string()),
            idempotency_key: Some("m2-reference-slice-full-aggregate-key".to_string()),
            expected_revision: Some(expected_revision),
        };
        crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect("commit the named reference-slice transition");

        let persisted = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read persisted full aggregate");
        let snapshot = crate::workbench_sqlite_repository::
            m2_workflow_state_sidecar_snapshot_from_projection(
                &fixture.project_root,
                &fixture.workflow_id,
                expected_revision + 1,
                &persisted,
            )
            .expect("derive canonical full aggregate snapshot");
        assert_eq!(
            snapshot.object_ref,
            format!("workflow_state:{}:{}", fixture.project_root, fixture.workflow_id)
        );

        // A different item and node in the same workflow must affect the
        // authoritative aggregate hash; this prevents a local current-item
        // pair from masquerading as the workflow-state snapshot.
        let mut changed_same_workflow = persisted.clone();
        let mut another_node = changed_same_workflow["nodes"]
            .as_array()
            .and_then(|nodes| nodes.first())
            .cloned()
            .expect("fixture node");
        another_node["node_id"] = Value::String(format!("{}:node:parallel", fixture.workflow_id));
        let mut another_item = changed_same_workflow["work_items"]
            .as_array()
            .and_then(|items| items.first())
            .cloned()
            .expect("fixture work item");
        another_item["work_item_id"] =
            Value::String(format!("{}:work-item:parallel", fixture.workflow_id));
        another_item["node_id"] = another_node["node_id"].clone();
        changed_same_workflow["nodes"]
            .as_array_mut()
            .expect("fixture node array")
            .push(another_node);
        changed_same_workflow["work_items"]
            .as_array_mut()
            .expect("fixture work item array")
            .push(another_item);
        let changed_snapshot = crate::workbench_sqlite_repository::
            m2_workflow_state_sidecar_snapshot_from_projection(
                &fixture.project_root,
                &fixture.workflow_id,
                expected_revision + 1,
                &changed_same_workflow,
            )
            .expect("derive changed full aggregate snapshot");
        assert_ne!(
            changed_snapshot.snapshot_hash, snapshot.snapshot_hash,
            "another named-slice item/node must change the authoritative aggregate hash"
        );

        // The same semantic record with a different object-key insertion
        // order is canonicalized to the same snapshot hash.
        let mut reordered = persisted.clone();
        let work_item = reordered["work_items"]
            .as_array_mut()
            .and_then(|items| items.first_mut())
            .expect("fixture work item for key reorder");
        let original = work_item.as_object().expect("work item object").clone();
        let mut reverse_order = serde_json::Map::new();
        for (key, value) in original.iter().rev() {
            reverse_order.insert(key.clone(), value.clone());
        }
        *work_item = Value::Object(reverse_order);
        let reordered_snapshot = crate::workbench_sqlite_repository::
            m2_workflow_state_sidecar_snapshot_from_projection(
                &fixture.project_root,
                &fixture.workflow_id,
                expected_revision + 1,
                &reordered,
            )
            .expect("derive reordered full aggregate snapshot");
        assert_eq!(
            reordered_snapshot.snapshot_hash, snapshot.snapshot_hash,
            "key-only JSON reordering must not change the canonical aggregate hash"
        );

        let connection = Connection::open(&fixture.config.db_path).expect("open fixture DB write");
        let (object_ref, revision, source_watermark, stored_hash): (String, i64, String, String) =
            connection
                .query_row(
                    "SELECT object_ref, object_revision, source_watermark, snapshot_hash
                     FROM current_snapshots
                     WHERE object_ref = ?1 AND projector_id = 'workflow_projector'",
                    [snapshot.object_ref.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .expect("read durable authoritative snapshot");
        assert_eq!(revision, expected_revision + 1);
        assert_eq!(stored_hash, snapshot.snapshot_hash);
        let rows = connection
            .execute(
                "UPDATE projection_checkpoints
                 SET last_event_id = 'event:stale-before-checkpoint',
                     source_watermark = 'event:stale-before-checkpoint',
                     status = 'ADVANCING'
                 WHERE projector_id = ?1 AND projector_version = ?2",
                params![
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
                ],
            )
            .expect("model crash after projection before checkpoint");
        assert_eq!(rows, 1, "fixture has one named-slice checkpoint");
        drop(connection);
        let json_before_restart = fs::read(&fixture.state_path).expect("read projection before restart");
        let owner_ledger_before_restart = m2_workflow_state_ledger_counts(&fixture.config);

        clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
        initialize_for_startup(&fixture.state_path)
            .expect("green persisted aggregate repairs its stale checkpoint on restart");

        assert_eq!(
            fs::read(&fixture.state_path).expect("read projection after checkpoint repair"),
            json_before_restart,
            "checkpoint recovery must not rewrite the persisted JSON projection"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            owner_ledger_before_restart,
            "checkpoint recovery must not duplicate owner receipt/event/audit/outbox/snapshot facts"
        );
        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open checkpoint DB after restart");
        let checkpoint: (Option<String>, String, String) = connection
            .query_row(
                "SELECT last_event_id, source_watermark, status
                 FROM projection_checkpoints WHERE projector_id = ?1 AND projector_version = ?2",
                params![
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID,
                    crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_VERSION,
                ],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read repaired checkpoint");
        assert_eq!(checkpoint.0.as_deref(), Some(source_watermark.as_str()));
        assert_eq!(checkpoint.1, source_watermark);
        assert_eq!(checkpoint.2, "CAUGHT_UP");
        assert!(
            reconcile_db_vs_json(&fixture.config)
                .expect("reconcile rebuilt checkpoint fixture")
                .is_green(),
            "checkpoint recovery must preserve DB-primary/JSON parity"
        );
        assert_eq!(object_ref, snapshot.object_ref);
    }

    #[test]
    fn m2_reference_slice_persisted_projection_tamper_is_fail_closed_without_checkpoint_or_db_rewrite() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-persisted-projection-tamper");
        let expected_revision = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository")
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read authoritative workflow revision");
        let request = crate::WorkItemStateUpdateRequest {
            project_root: fixture.project_root.clone(),
            work_item_id: fixture.work_item_id.clone(),
            next_state: "running".to_string(),
            command_id: Some("m2-reference-slice-persisted-projection-tamper-command".to_string()),
            idempotency_key: Some("m2-reference-slice-persisted-projection-tamper-key".to_string()),
            expected_revision: Some(expected_revision),
        };
        crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect("create the authoritative snapshot and persisted projection");

        let persisted_before_tamper = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read persisted canonical projection before tamper");
        let equivalent_hash_left = crate::workbench_sqlite_repository::
            m2_workflow_state_sidecar_snapshot_from_projection(
                &fixture.project_root,
                &fixture.workflow_id,
                expected_revision + 1,
                &persisted_before_tamper,
            )
            .expect("derive canonical persisted aggregate")
            .snapshot_hash;
        let mut reordered_projection = persisted_before_tamper.clone();
        let work_item = reordered_projection["work_items"]
            .as_array_mut()
            .and_then(|items| items.first_mut())
            .expect("fixture work item for canonical reorder");
        let original = work_item.as_object().expect("work item object").clone();
        let mut reverse_order = serde_json::Map::new();
        for (key, value) in original.iter().rev() {
            reverse_order.insert(key.clone(), value.clone());
        }
        *work_item = Value::Object(reverse_order);
        let equivalent_hash_right = crate::workbench_sqlite_repository::
            m2_workflow_state_sidecar_snapshot_from_projection(
                &fixture.project_root,
                &fixture.workflow_id,
                expected_revision + 1,
                &reordered_projection,
            )
            .expect("derive reordered canonical aggregate")
            .snapshot_hash;
        assert_eq!(
            equivalent_hash_left, equivalent_hash_right,
            "canonical snapshot hashing must not depend on JSON object key order"
        );

        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open authoritative DB before disk tamper");
        let checkpoint_before: (String, String, Option<String>) = connection
            .query_row(
                "SELECT status, source_watermark, last_event_id
                 FROM projection_checkpoints WHERE projector_id = ?1",
                [crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read checkpoint before disk tamper");
        let snapshot_before: String = connection
            .query_row(
                "SELECT snapshot_hash FROM current_snapshots
                 WHERE object_ref = ?1 AND projector_id = 'workflow_projector'",
                [format!("workflow_state:{}:{}", fixture.project_root, fixture.workflow_id)],
                |row| row.get(0),
            )
            .expect("read authoritative snapshot before disk tamper");
        drop(connection);
        let db_before = fs::read(&fixture.config.db_path).expect("read DB before disk tamper");

        let mut persisted_projection = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read persisted JSON projection");
        let item = persisted_projection["work_items"]
            .as_array_mut()
            .and_then(|items| {
                items.iter_mut().find(|item| {
                    item.get("work_item_id").and_then(Value::as_str)
                        == Some(fixture.work_item_id.as_str())
                })
            })
            .expect("find persisted work item to tamper");
        item["title"] = Value::String("tampered after authoritative commit".to_string());
        crate::write_validated_workflow_state(&fixture.state_path, &persisted_projection)
            .expect("write structurally valid but semantically divergent projection");
        let tampered_json = fs::read(&fixture.state_path).expect("read tampered projection bytes");

        clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
        let error = initialize_for_startup(&fixture.state_path)
            .expect_err("persisted projection tamper must block DB-primary startup");
        assert!(error.contains("db_primary_projection_blocked"), "{error}");
        assert!(
            error.contains("hash_mismatches") || error.contains("json_leading"),
            "{error}"
        );
        assert_eq!(
            fs::read(&fixture.config.db_path).expect("read DB after rejected tamper"),
            db_before,
            "a JSON projection tamper must not rewrite authoritative DB state"
        );
        assert_eq!(
            fs::read(&fixture.state_path).expect("read JSON after rejected tamper"),
            tampered_json,
            "fail-closed startup must not reverse-overwrite the disk projection"
        );

        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open DB after rejected tamper");
        let checkpoint_after: (String, String, Option<String>) = connection
            .query_row(
                "SELECT status, source_watermark, last_event_id
                 FROM projection_checkpoints WHERE projector_id = ?1",
                [crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read checkpoint after rejected tamper");
        let snapshot_after: String = connection
            .query_row(
                "SELECT snapshot_hash FROM current_snapshots
                 WHERE object_ref = ?1 AND projector_id = 'workflow_projector'",
                [format!("workflow_state:{}:{}", fixture.project_root, fixture.workflow_id)],
                |row| row.get(0),
            )
            .expect("read authoritative snapshot after rejected tamper");
        assert_eq!(checkpoint_after, checkpoint_before, "tamper may not advance checkpoint");
        assert_eq!(snapshot_after, snapshot_before, "tamper may not replace authoritative snapshot");
    }

    #[test]
    fn m2_reference_slice_r4_fake_external_adapter_state_machine_is_real_uow_only() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-r4-fake-external-adapter");
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository");
        const FIRST_CLAIM_AT: i64 = 1_800_000_000_000;

        // The profile is deliberately isolated.  It proves the frozen outbox
        // state chain through the production SQLite repository/UoW without a
        // provider, normal-product side effect, or JSON projection outbox.
        let success_payload_hash = sha256_hex("m2-r4-fake-external-success");
        let success = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_declare_success",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::declare_m2_r4_fake_external_adapter_effect_in_transaction(
                        transaction,
                        &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterDeclaration {
                            command_id: "cmd:m2-r4-fake-external-success",
                            idempotency_key: "idem:m2-r4-fake-external-success",
                            actor_id: "m2-reference-slice-test",
                            scope_ref: "workflow-state-sidecar:m2-r4-acceptance",
                            subject_ref: "work-item:m2-r4-acceptance",
                            payload_hash: &success_payload_hash,
                            cancel_before_available: false,
                        },
                        FIRST_CLAIM_AT,
                    )
                },
            )
            .expect("declare isolated fake external effect")
            .0;
        assert_eq!(success.status, "AVAILABLE");

        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open declared effect DB read only");
        let (available_status, pending_owner): (String, String) = connection
            .query_row(
                "SELECT outbox_items.status, command_receipts.status
                 FROM outbox_items JOIN command_receipts
                   ON command_receipts.receipt_id = outbox_items.owning_command_receipt_ref
                 WHERE outbox_item_id = ?1",
                [&success.outbox_item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read declaration state chain");
        assert_eq!(available_status, "AVAILABLE");
        assert_eq!(pending_owner, "EXTERNAL_PENDING");
        drop(connection);

        let lease = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_claim_success",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &success.outbox_item_id,
                    FIRST_CLAIM_AT,
                ),
            )
            .expect("claim isolated fake effect")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
            other => panic!("new fake effect must lease: {other:?}"),
        };
        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open leased effect DB read only");
        let expires_at: Option<String> = connection
            .query_row(
                "SELECT expires_at FROM outbox_items WHERE outbox_item_id = ?1",
                [&success.outbox_item_id],
                |row| row.get(0),
            )
            .expect("read frozen 300s lease expiry");
        assert_eq!(
            expires_at.as_deref(),
            Some(
                (FIRST_CLAIM_AT
                    + crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS)
                    .to_string()
                    .as_str()
            ),
            "the R4-only adapter must use the frozen 300s lease"
        );
        drop(connection);
        for extension_at in [FIRST_CLAIM_AT + 1, FIRST_CLAIM_AT + 2] {
            let extended = repository
                .with_immediate_transaction(
                    "m2_r4_fake_external_adapter_extend_success",
                    None,
                    |transaction| crate::workbench_sqlite_repository::extend_m2_r4_fake_external_adapter_lease_in_transaction(
                        transaction,
                        &lease,
                        extension_at,
                    ),
                )
                .expect("the first two 300s lease extensions are permitted")
                .0;
            assert_eq!(extended, lease, "extension preserves the exact lease identity");
        }
        let extension_limit = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_extend_limit",
                None,
                |transaction| crate::workbench_sqlite_repository::extend_m2_r4_fake_external_adapter_lease_in_transaction(
                    transaction,
                    &lease,
                    FIRST_CLAIM_AT + 3,
                ),
            )
            .expect_err("a third extension is rejected without another outbox transition");
        assert!(extension_limit.contains("lease_extension_limit"), "{extension_limit}");
        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open extended lease DB read only");
        let (extension_count, extended_expiry): (i64, String) = connection
            .query_row(
                "SELECT lease_extension_count, expires_at FROM outbox_items WHERE outbox_item_id = ?1",
                [&success.outbox_item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read exact extension state");
        assert_eq!(
            extension_count,
            crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_LEASE_EXTENSIONS,
        );
        assert_eq!(
            extended_expiry,
            (FIRST_CLAIM_AT + 2 + crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS).to_string(),
        );
        drop(connection);
        let result_hash = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_deliver_success",
                None,
                |transaction| crate::workbench_sqlite_repository::deliver_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &lease,
                    FIRST_CLAIM_AT + 4,
                ),
            )
            .expect("deliver deterministic local fake effect")
            .0;
        // Admission is re-resolved from the owning receipt/outbox, rather
        // than accepted from a caller supplied result envelope.  Every wrong
        // semantic binding must reject before it can create a result receipt,
        // event, audit or mutate the owning effect.
        let ledger_counts = || -> [i64; 4] {
            let connection = Connection::open_with_flags(
                &fixture.config.db_path,
                OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .expect("open result-admission DB read only");
            [
                connection
                    .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| row.get(0))
                    .expect("count receipts"),
                connection
                    .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
                    .expect("count events"),
                connection
                    .query_row("SELECT COUNT(*) FROM audit_records", [], |row| row.get(0))
                    .expect("count audit"),
                connection
                    .query_row("SELECT COUNT(*) FROM outbox_items", [], |row| row.get(0))
                    .expect("count outbox"),
            ]
        };
        for (case_name, actor_id, scope_ref, effect_id, correlation_id) in [
            (
                "wrong-actor",
                "m2-r4-forged-actor",
                success.scope_ref.as_str(),
                success.effect_id.as_str(),
                success.correlation_id.as_str(),
            ),
            (
                "wrong-scope",
                success.actor_id.as_str(),
                "workflow-state-sidecar:forged-scope",
                success.effect_id.as_str(),
                success.correlation_id.as_str(),
            ),
            (
                "wrong-effect",
                success.actor_id.as_str(),
                success.scope_ref.as_str(),
                "m2-r4-forged-effect",
                success.correlation_id.as_str(),
            ),
            (
                "wrong-correlation",
                success.actor_id.as_str(),
                success.scope_ref.as_str(),
                success.effect_id.as_str(),
                "m2-r4-forged-correlation",
            ),
        ] {
            let before = ledger_counts();
            let error = repository
                .with_immediate_transaction(
                    "m2_r4_fake_external_adapter_result_binding_rejected",
                    None,
                    |transaction| {
                        crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                            transaction,
                            &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand {
                                command_id: "cmd:m2-r4-fake-external-result",
                                idempotency_key: "idem:m2-r4-fake-external-result",
                                outbox_item_id: success.outbox_item_id.as_str(),
                                result_hash: result_hash.as_str(),
                                owning_command_id: success.owning_command_id.as_str(),
                                owning_receipt_id: success.owning_receipt_id.as_str(),
                                effect_id,
                                envelope: crate::workbench_sqlite_repository::M2R4NormalizedCommandEnvelope {
                                    actor_id,
                                    scope_ref,
                                    current_object_ref: success.current_object_ref.as_str(),
                                    channel: crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_CHANNEL,
                                    permission_ref: crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_PERMISSION,
                                    admission_ref: crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_RESULT_ADMISSION,
                                    correlation_id,
                                    causation_id: success.causation_id.as_str(),
                                },
                            },
                            FIRST_CLAIM_AT + 5,
                        )
                    },
                )
                .expect_err("wrong result envelope must be denied before write");
            assert!(
                error.contains("m2_r4_fake_external_result"),
                "{case_name}:{error}"
            );
            assert_eq!(ledger_counts(), before, "{case_name} may not mutate any ledger");
        }
        let result = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_result_success",
                None,
                |transaction| crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                    transaction,
                    &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand::for_owned_effect(
                        "cmd:m2-r4-fake-external-result",
                        "idem:m2-r4-fake-external-result",
                        &success,
                        &result_hash,
                    ),
                    FIRST_CLAIM_AT + 5,
                ),
            )
            .expect("record separately identified result command")
            .0;
        assert!(!result.replayed);
        let replay = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_result_replay",
                None,
                |transaction| crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                    transaction,
                    &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand::for_owned_effect(
                        "cmd:m2-r4-fake-external-result",
                        "idem:m2-r4-fake-external-result",
                        &success,
                        &result_hash,
                    ),
                    FIRST_CLAIM_AT + 6,
                ),
            )
            .expect("exact result command replay")
            .0;
        assert!(replay.replayed);
        assert_eq!(replay.receipt_id, result.receipt_id);
        let forged_result_hash = sha256_hex("m2-r4-fake-external-forged-result");
        let divergent_error = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_result_divergent",
                None,
                |transaction| crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                    transaction,
                    &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand::for_owned_effect(
                        "cmd:m2-r4-fake-external-result",
                        "idem:m2-r4-fake-external-result",
                        &success,
                        &forged_result_hash,
                    ),
                    FIRST_CLAIM_AT + 7,
                ),
            )
            .expect_err("divergent result may not overwrite an accepted result");
        assert!(divergent_error.contains("idempotency_conflict"), "{divergent_error}");

        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open result DB read only");
        let (result_status, owner_status, result_receipts): (String, String, i64) = connection
            .query_row(
                "SELECT outbox_items.status, owning.status,
                        (SELECT COUNT(*) FROM command_receipts WHERE command_id = 'cmd:m2-r4-fake-external-result')
                 FROM outbox_items JOIN command_receipts AS owning
                   ON owning.receipt_id = outbox_items.owning_command_receipt_ref
                 WHERE outbox_items.outbox_item_id = ?1",
                [&success.outbox_item_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read completed state chain");
        assert_eq!(result_status, "RESULT_RECEIVED");
        assert_eq!(owner_status, "EXTERNAL_RESULT");
        assert_eq!(result_receipts, 1, "replay must not grow result receipts");
        drop(connection);

        let retry_payload_hash = sha256_hex("m2-r4-fake-external-retry");
        let retry = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_declare_retry",
                None,
                |transaction| crate::workbench_sqlite_repository::declare_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterDeclaration {
                        command_id: "cmd:m2-r4-fake-external-retry",
                        idempotency_key: "idem:m2-r4-fake-external-retry",
                        actor_id: "m2-reference-slice-test",
                        scope_ref: "workflow-state-sidecar:m2-r4-acceptance",
                        subject_ref: "work-item:m2-r4-acceptance",
                        payload_hash: &retry_payload_hash,
                        cancel_before_available: false,
                    },
                    FIRST_CLAIM_AT,
                ),
            )
            .expect("declare retry effect")
            .0;
        let retry_lease_one = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_claim_retry_one",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(transaction, &retry.outbox_item_id, FIRST_CLAIM_AT),
            )
            .expect("claim retry effect")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
            other => panic!("retry effect must first lease: {other:?}"),
        };
        let expired_available = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_expire_retry_one",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &retry_lease_one.outbox_item_id,
                    FIRST_CLAIM_AT
                        + crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS,
                ),
            )
            .expect("expired 300s lease returns to AVAILABLE without spending retry budget")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::LeaseExpiredAvailable { outbox_item_id } => outbox_item_id,
            other => panic!("expired first lease must return AVAILABLE: {other:?}"),
        };
        assert_eq!(expired_available, retry.outbox_item_id);
        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open expired lease DB read only");
        let (expired_status, expiry_attempts): (String, i64) = connection
            .query_row(
                "SELECT status, attempt_count FROM outbox_items WHERE outbox_item_id = ?1",
                [&retry.outbox_item_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read released expiry state");
        assert_eq!(expired_status, "AVAILABLE");
        assert_eq!(expiry_attempts, 0, "lease expiry is not a delivery failure");
        drop(connection);
        let retry_lease_two = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_claim_retry_two",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &retry.outbox_item_id,
                    FIRST_CLAIM_AT + crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS + 1,
                ),
            )
            .expect("released effect is claimable again")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
            other => panic!("released effect must lease: {other:?}"),
        };
        let retry_two_at = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_fail_retry_two",
                None,
                |transaction| crate::workbench_sqlite_repository::fail_m2_r4_fake_external_adapter_delivery_in_transaction(
                    transaction,
                    &retry_lease_two,
                    FIRST_CLAIM_AT + crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS + 2,
                    "fixture_retry_one",
                ),
            )
            .expect("first explicit delivery failure schedules retry")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::RetryScheduled { retry_not_before, .. } => retry_not_before,
            other => panic!("first delivery failure must retry: {other:?}"),
        };
        assert!(
            retry_two_at
                > FIRST_CLAIM_AT
                    + crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_LEASE_MS
                    + 1_000,
            "delivery failure retry is exponential plus deterministic jitter"
        );
        let retry_lease_three = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_claim_retry_three",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(transaction, &retry.outbox_item_id, retry_two_at),
            )
            .expect("second lease after scheduled retry")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
            other => panic!("second attempt must lease before poison: {other:?}"),
        };
        let poison = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_fail_retry_two",
                None,
                |transaction| crate::workbench_sqlite_repository::fail_m2_r4_fake_external_adapter_delivery_in_transaction(transaction, &retry_lease_three, retry_two_at + 1, "fixture_retry_two"),
            )
            .expect("second delivery failure schedules the last retry")
            .0;
        let retry_three_at = match poison {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::RetryScheduled { retry_not_before, .. } => retry_not_before,
            other => panic!("second delivery failure must schedule final retry: {other:?}"),
        };
        let retry_lease_four = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_claim_retry_four",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(transaction, &retry.outbox_item_id, retry_three_at),
            )
            .expect("third delivery attempt leases after scheduled retry")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
            other => panic!("third delivery attempt must lease before poison: {other:?}"),
        };
        let poison = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_fail_retry_three",
                None,
                |transaction| crate::workbench_sqlite_repository::fail_m2_r4_fake_external_adapter_delivery_in_transaction(transaction, &retry_lease_four, retry_three_at + 1, "fixture_retry_three"),
            )
            .expect("third delivery failure poisons")
            .0;
        assert!(matches!(poison, crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Poisoned { .. }));
        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open poison DB read only");
        let (poison_status, attempts, poisoned_owner): (String, i64, String) = connection
            .query_row(
                "SELECT outbox_items.status, outbox_items.attempt_count, command_receipts.status
                 FROM outbox_items JOIN command_receipts
                   ON command_receipts.receipt_id = outbox_items.owning_command_receipt_ref
                 WHERE outbox_items.outbox_item_id = ?1",
                [&retry.outbox_item_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read poison state");
        assert_eq!(poison_status, "POISON");
        assert_eq!(attempts, crate::workbench_sqlite_repository::M2_R4_FAKE_EXTERNAL_ADAPTER_MAX_ATTEMPTS);
        assert_eq!(poisoned_owner, "PROJECTION_DEGRADED");
        drop(connection);

        let cancelled_payload_hash = sha256_hex("m2-r4-fake-external-cancelled");
        let cancelled = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_declare_cancelled",
                None,
                |transaction| crate::workbench_sqlite_repository::declare_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterDeclaration {
                        command_id: "cmd:m2-r4-fake-external-cancelled",
                        idempotency_key: "idem:m2-r4-fake-external-cancelled",
                        actor_id: "m2-reference-slice-test",
                        scope_ref: "workflow-state-sidecar:m2-r4-acceptance",
                        subject_ref: "work-item:m2-r4-acceptance",
                        payload_hash: &cancelled_payload_hash,
                        cancel_before_available: true,
                    },
                    FIRST_CLAIM_AT,
                ),
            )
            .expect("declare cancelled branch")
            .0;
        assert_eq!(cancelled.status, "CANCELLED");
    }

    #[test]
    fn m2_reference_slice_r4_fake_external_adapter_concurrent_result_is_one_receipt() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-r4-fake-external-concurrent-result");
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository");
        const NOW: i64 = 1_800_000_100_000;
        let payload_hash = sha256_hex("m2-r4-fake-external-concurrent-result");
        let declaration = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_concurrent_declare",
                None,
                |transaction| crate::workbench_sqlite_repository::declare_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterDeclaration {
                        command_id: "cmd:m2-r4-fake-external-concurrent-owner",
                        idempotency_key: "idem:m2-r4-fake-external-concurrent-owner",
                        actor_id: "m2-reference-slice-test",
                        scope_ref: "workflow-state-sidecar:m2-r4-acceptance",
                        subject_ref: "work-item:m2-r4-acceptance",
                        payload_hash: &payload_hash,
                        cancel_before_available: false,
                    },
                    NOW,
                ),
            )
            .expect("declare concurrent result fixture")
            .0;
        let lease = match repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_concurrent_claim",
                None,
                |transaction| crate::workbench_sqlite_repository::claim_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &declaration.outbox_item_id,
                    NOW,
                ),
            )
            .expect("claim concurrent result fixture")
            .0
        {
            crate::workbench_sqlite_repository::M2R4FakeExternalAdapterClaim::Leased(lease) => lease,
            other => panic!("concurrent result fixture must lease: {other:?}"),
        };
        let result_hash = repository
            .with_immediate_transaction(
                "m2_r4_fake_external_adapter_concurrent_deliver",
                None,
                |transaction| crate::workbench_sqlite_repository::deliver_m2_r4_fake_external_adapter_effect_in_transaction(
                    transaction,
                    &lease,
                    NOW + 1,
                ),
            )
            .expect("deliver concurrent result fixture")
            .0;

        let start = Arc::new(Barrier::new(3));
        let hash_one = result_hash.clone();
        let declaration_one = declaration.clone();
        let repository_one = repository.clone();
        let start_one = Arc::clone(&start);
        let hash_two = result_hash.clone();
        let declaration_two = declaration.clone();
        let repository_two = repository.clone();
        let start_two = Arc::clone(&start);
        let (first, second) = std::thread::scope(|scope| {
            let first = scope.spawn(move || {
                start_one.wait();
                repository_one
                    .with_immediate_transaction(
                        "m2_r4_fake_external_adapter_concurrent_result_one",
                        None,
                        |transaction| crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                            transaction,
                            &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand::for_owned_effect(
                                "cmd:m2-r4-fake-external-concurrent-result",
                                "idem:m2-r4-fake-external-concurrent-result",
                                &declaration_one,
                                &hash_one,
                            ),
                            NOW + 2,
                        ),
                    )
                    .map(|result| result.0)
            });
            let second = scope.spawn(move || {
                start_two.wait();
                repository_two
                    .with_immediate_transaction(
                        "m2_r4_fake_external_adapter_concurrent_result_two",
                        None,
                        |transaction| crate::workbench_sqlite_repository::record_m2_r4_fake_external_adapter_result_command_in_transaction(
                            transaction,
                            &crate::workbench_sqlite_repository::M2R4FakeExternalAdapterResultCommand::for_owned_effect(
                                "cmd:m2-r4-fake-external-concurrent-result",
                                "idem:m2-r4-fake-external-concurrent-result",
                                &declaration_two,
                                &hash_two,
                            ),
                            NOW + 2,
                        ),
                    )
                    .map(|result| result.0)
            });
            start.wait();
            (
                first.join().expect("first concurrent result thread joined"),
                second.join().expect("second concurrent result thread joined"),
            )
        });
        let first = first.expect("first concurrent result command accepted");
        let second = second.expect("second concurrent result command replayed");
        assert_eq!(first.receipt_id, second.receipt_id);
        assert_ne!(first.replayed, second.replayed, "one result writes and one replays");
        let connection = Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open concurrent result DB read only");
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM command_receipts
                 WHERE command_id = 'cmd:m2-r4-fake-external-concurrent-result'",
                [],
                |row| row.get(0),
            )
            .expect("count concurrent result receipts");
        assert_eq!(count, 1, "concurrent identical result commands have one durable receipt");
    }

    // Retained only as source-history context for the former JSON-projection
    // experiment.  JSON is now an internal rebuildable projection, so this
    // test is intentionally not compiled or counted as a DAT-004 result path.
    #[cfg(any())]
    fn legacy_json_projection_outbox_state_machine_retained_as_non_authoritative_fixture() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-outbox-state-machine");
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository");
        let command = crate::m2_workflow_state::UpdateWorkItemStateCommand {
            command_id: format!("cmd-{}-running-outbox-state-machine", fixture.work_item_id),
            idempotency_key: "m2-reference-slice-outbox-state-machine".to_string(),
            actor_id: "m2-reference-slice-test".to_string(),
            scope_ref: format!("workflow:{}", fixture.project_root),
            project_id: fixture.project_root.clone(),
            workflow_id: fixture.workflow_id.clone(),
            work_item_id: fixture.work_item_id.clone(),
            expected_revision: None,
            new_status: Some(crate::m2_workflow_state::WorkItemStatus::Running),
            new_state_json: None,
        };
        let (outbox_item_id, receipt_id) = repository
            .with_immediate_transaction(
                "m2_reference_slice_outbox_state_machine_seed",
                None,
                |transaction| {
                    let result = crate::m2_update_work_item_state::update_work_item_state_m2_with_transaction(
                        transaction,
                        command.clone(),
                    )
                    .map_err(crate::workbench_sqlite_repository::RepositoryMutationError::Message)?;
                    let outbox_item = result.outbox_item.ok_or_else(|| {
                        crate::workbench_sqlite_repository::RepositoryMutationError::Message(
                            "m2_reference_slice_outbox_missing".to_string(),
                        )
                    })?;
                    Ok((outbox_item.outbox_item_id, result.receipt.receipt_id))
                },
            )
            .expect("commit the exact reference-slice outbox declaration")
            .0;

        let cancellation_error = repository
            .with_immediate_transaction(
                "m2_reference_slice_required_projection_cancellation",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::reject_m2_sidecar_outbox_cancellation_in_transaction(
                        transaction,
                        &outbox_item_id,
                    )
                },
            )
            .expect_err("the mandatory local projection must not be silently cancelled");
        assert!(
            cancellation_error.contains("m2_sidecar_outbox_cancellation_forbidden_required_projection"),
            "{cancellation_error}"
        );

        let connection =
            Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open M2 fixture DB read only");
        let available_status: String = connection
            .query_row(
                "SELECT status FROM outbox_items WHERE outbox_item_id = ?1",
                [outbox_item_id.as_str()],
                |row| row.get(0),
            )
            .expect("read cancellation-refused outbox state");
        assert_eq!(available_status, "AVAILABLE");
        drop(connection);

        const FIRST_CLAIM_AT: i64 = 1_800_000_000_000;
        let first_lease = match repository
            .with_immediate_transaction(
                "m2_reference_slice_first_claim",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::claim_m2_sidecar_outbox_in_transaction(
                        transaction,
                        &outbox_item_id,
                        FIRST_CLAIM_AT,
                    )
                },
            )
            .expect("claim the exact reference-slice effect")
            .0
        {
            crate::workbench_sqlite_repository::WorkflowStateProjectionClaim::Leased(lease) => lease,
            crate::workbench_sqlite_repository::WorkflowStateProjectionClaim::Poisoned { .. } => {
                panic!("a new reference-slice effect must not be poisoned")
            }
        };

        // A second claimant after the lease interval must durably consume the
        // expired attempt and acquire the one retry lease, rather than rolling
        // the expiry transition back with an error return.
        let expired_reclaim = match repository
            .with_immediate_transaction(
                "m2_reference_slice_expired_lease_reclaim",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::claim_m2_sidecar_outbox_in_transaction(
                        transaction,
                        &outbox_item_id,
                        FIRST_CLAIM_AT + 120_001,
                    )
                },
            )
            .expect("expired lease must enter the bounded retry path")
            .0
        {
            crate::workbench_sqlite_repository::WorkflowStateProjectionClaim::Leased(lease) => lease,
            crate::workbench_sqlite_repository::WorkflowStateProjectionClaim::Poisoned { .. } => {
                panic!("the first expired attempt must still be retryable")
            }
        };
        assert_ne!(first_lease.lease_token, expired_reclaim.lease_token);

        let retry_status = repository
            .with_immediate_transaction(
                "m2_reference_slice_second_failed_delivery",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::retry_m2_sidecar_outbox_in_transaction(
                        transaction,
                        &expired_reclaim,
                        FIRST_CLAIM_AT + 120_002,
                        "fixture_second_delivery_failure",
                    )
                },
            )
            .expect("record the second bounded failed delivery")
            .0;
        assert_eq!(retry_status, "RETRY_WAIT");

        let connection =
            Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open retry fixture DB read only");
        let retry_not_before: String = connection
            .query_row(
                "SELECT next_retry_not_before FROM outbox_items WHERE outbox_item_id = ?1",
                [outbox_item_id.as_str()],
                |row| row.get(0),
            )
            .expect("read exact retry schedule");
        drop(connection);
        let retry_claim_at = retry_not_before
            .parse::<i64>()
            .expect("retry schedule must be an epoch millisecond");
        let final_lease = match repository
            .with_immediate_transaction(
                "m2_reference_slice_final_retry_claim",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::claim_m2_sidecar_outbox_in_transaction(
                        transaction,
                        &outbox_item_id,
                        retry_claim_at,
                    )
                },
            )
            .expect("claim the final bounded retry")
            .0
        {
            crate::workbench_sqlite_repository::WorkflowStateProjectionClaim::Leased(lease) => lease,
            crate::workbench_sqlite_repository::WorkflowStateProjectionClaim::Poisoned { .. } => {
                panic!("the third attempt begins as a lease before its failure is recorded")
            }
        };
        let poison_status = repository
            .with_immediate_transaction(
                "m2_reference_slice_poison_after_bounded_failures",
                None,
                |transaction| {
                    crate::workbench_sqlite_repository::retry_m2_sidecar_outbox_in_transaction(
                        transaction,
                        &final_lease,
                        retry_claim_at + 1,
                        "fixture_final_delivery_failure",
                    )
                },
            )
            .expect("persist poison after the bounded retry limit")
            .0;
        assert_eq!(poison_status, "POISON");

        let connection =
            Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open final outbox fixture DB read only");
        let (status, attempt_count, receipt_status): (String, i64, String) = connection
            .query_row(
                "SELECT outbox_items.status, outbox_items.attempt_count, command_receipts.status
                 FROM outbox_items
                 JOIN command_receipts
                   ON command_receipts.receipt_id = outbox_items.owning_command_receipt_ref
                 WHERE outbox_items.outbox_item_id = ?1",
                [outbox_item_id.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read bounded poison outcome");
        assert_eq!(status, "POISON");
        assert_eq!(attempt_count, 3);
        assert_eq!(receipt_status, "PROJECTION_DEGRADED");
        let (checkpoint_status, checkpoint_last_event, checkpoint_error_receipt):
            (String, Option<String>, Option<String>) = connection
            .query_row(
                "SELECT status, last_event_id, error_receipt_ref
                 FROM projection_checkpoints WHERE projector_id = ?1",
                [crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_PROJECTOR_ID],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("record degraded checkpoint for the exact failed sidecar");
        assert_eq!(checkpoint_status, "DEGRADED");
        assert_eq!(checkpoint_last_event, None);
        assert_eq!(checkpoint_error_receipt.as_deref(), Some(receipt_id.as_str()));
    }

    fn assert_m2_workflow_state_sidecar_quarantine_case(
        label: &str,
        expected_reason: &str,
        raw_marker: Option<&str>,
        mutate: impl FnOnce(&std::path::Path),
    ) {
        let fixture = db_primary_fixture(label);
        let original = fs::read(&fixture.state_path).expect("retain the pre-quarantine sidecar");
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [0, 0, 0, 0, 0],
            "case starts without M2 ledger rows"
        );
        mutate(&fixture.state_path);
        let observed_bytes = fs::read(&fixture.state_path).expect("read changed sidecar bytes");
        let observed_sha256 = sha256_hex_bytes(&observed_bytes);

        let blocked = initialize_for_startup(&fixture.state_path)
            .expect_err("unknown/corrupt/sensitive/unjoinable sidecar must fail closed");
        assert!(
            blocked.contains(&format!(
                "m2_workflow_state_sidecar_quarantined:{expected_reason}:"
            )),
            "{blocked}"
        );
        assert_db_primary_health_blocked(&fixture.state_path, "m2_workflow_state_sidecar_quarantined");
        let manifest = m2_workflow_state_sidecar_quarantine_manifest(&fixture.config)
            .expect("value-free quarantine export");
        let [entry] = manifest.as_slice() else {
            panic!("expected one exact quarantine entry: {manifest:?}");
        };
        assert_eq!(entry.reason_code, expected_reason);
        assert_eq!(
            entry.source_ref,
            format!("workflow-state-sidecar:sha256:{observed_sha256}")
        );
        assert_eq!(
            entry.scope_ref,
            crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_QUARANTINE_SCOPE
        );
        assert_eq!(entry.resolution_state, "PENDING");
        assert!(
            !format!("{entry:?}").contains(&fixture.state_path.display().to_string()),
            "value-free manifest must not expose the filesystem path"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [0, 1, 1, 0, 0],
            "quarantine records only one scrubbed event and audit, never a domain/outbox/snapshot mutation"
        );

        let replay_blocked = initialize_for_startup(&fixture.state_path)
            .expect_err("the same unresolved source remains fail closed");
        assert!(replay_blocked.contains(expected_reason), "{replay_blocked}");
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [0, 1, 1, 0, 0],
            "same input replay must not grow quarantine evidence"
        );

        if let Some(raw_marker) = raw_marker {
            let connection = Connection::open_with_flags(
                &fixture.config.db_path,
                OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .expect("open quarantine DB read only");
            let needle = format!("%{raw_marker}%");
            let references_with_marker: i64 = connection
                .query_row(
                    "SELECT
                       (SELECT COUNT(*) FROM unknown_quarantine
                        WHERE source_ref LIKE ?1 OR COALESCE(resolution_ref, '') LIKE ?1)
                     + (SELECT COUNT(*) FROM events
                        WHERE source_ref LIKE ?1 OR COALESCE(summary_ref, '') LIKE ?1
                           OR COALESCE(payload_ref, '') LIKE ?1)
                     + (SELECT COUNT(*) FROM audit_records
                        WHERE COALESCE(subject_ref, '') LIKE ?1
                           OR COALESCE(source_refs, '') LIKE ?1)",
                    [needle],
                    |row| row.get(0),
                )
                .expect("query value-free quarantine surfaces");
            assert_eq!(
                references_with_marker, 0,
                "sensitive input must never enter ordinary SQLite evidence"
            );
        }

        fs::write(&fixture.state_path, &original).expect("restore retained known-good fixture source");
        let unresolved = initialize_for_startup(&fixture.state_path)
            .expect_err("manual rebuild decision is required after an input quarantine");
        assert!(
            unresolved.contains("m2_workflow_state_sidecar_unresolved_quarantine"),
            "{unresolved}"
        );
        let rebuilt = rebuild_m2_workflow_state_sidecar_quarantine(
            &fixture.config,
            &entry.quarantine_id,
        )
        .expect("only a green replacement sidecar can rebuild the exact receipt");
        assert_eq!(rebuilt.quarantine_id, entry.quarantine_id);
        assert_eq!(rebuilt.resolution_state, "REBUILT");
        initialize_for_startup(&fixture.state_path)
            .expect("repaired value-free source may restart after exact rebuild receipt");
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [0, 2, 2, 0, 0],
            "rebuild contributes only its scrubbed event/audit receipt"
        );
        let exported = m2_workflow_state_sidecar_quarantine_manifest(&fixture.config)
            .expect("export rebuilt manifest");
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].resolution_state, "REBUILT");
    }

    #[test]
    fn m2_reference_slice_sidecar_quarantine_is_value_free_idempotent_and_rebuildable() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        assert_m2_workflow_state_sidecar_quarantine_case(
            "m2-reference-slice-quarantine-unknown",
            "UNKNOWN_INPUT",
            None,
            |state_path| {
                let mut value: Value = serde_json::from_slice(
                    &fs::read(state_path).expect("read unknown-input fixture"),
                )
                .expect("fixture JSON");
                value["future_unclassified_envelope"] = json!({"fixture_only": true});
                fs::write(state_path, serde_json::to_vec(&value).expect("serialize unknown fixture"))
                    .expect("write unknown fixture");
            },
        );
        assert_m2_workflow_state_sidecar_quarantine_case(
            "m2-reference-slice-quarantine-sensitive",
            "SENSITIVE_INPUT",
            Some("m2-quarantine-secret-never-persist"),
            |state_path| {
                let mut value: Value = serde_json::from_slice(
                    &fs::read(state_path).expect("read sensitive-input fixture"),
                )
                .expect("fixture JSON");
                value["credential_token"] = json!("m2-quarantine-secret-never-persist");
                fs::write(state_path, serde_json::to_vec(&value).expect("serialize sensitive fixture"))
                    .expect("write sensitive fixture");
            },
        );
        assert_m2_workflow_state_sidecar_quarantine_case(
            "m2-reference-slice-quarantine-unjoinable",
            "UNJOINABLE_REFERENCE_RECORD",
            None,
            |state_path| {
                let mut value: Value = serde_json::from_slice(
                    &fs::read(state_path).expect("read unjoinable-input fixture"),
                )
                .expect("fixture JSON");
                value["work_items"] = json!([{ "state": "running" }]);
                fs::write(state_path, serde_json::to_vec(&value).expect("serialize unjoinable fixture"))
                    .expect("write unjoinable fixture");
            },
        );
        assert_m2_workflow_state_sidecar_quarantine_case(
            "m2-reference-slice-quarantine-corrupt",
            "CORRUPT_INPUT",
            None,
            |state_path| {
                fs::write(state_path, b"{m2-corrupt-json")
                    .expect("write corrupt-input fixture");
            },
        );
    }

    #[test]
    fn m2_reference_slice_scratch_export_manifest_and_noop_rollback_preserve_sidecar() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-scratch-export-noop-rollback");
        let sidecar_before = fs::read(&fixture.state_path).expect("read retained sidecar before export");
        let database_before = fs::read(&fixture.config.db_path).expect("read DB before export");
        let source_ref = format!(
            "workflow-state-sidecar:sha256:{}",
            sha256_hex_bytes(&sidecar_before)
        );

        // Reuse the existing confirmed-DB dry-run exporter rather than adding
        // another M2 export surface.  Its return is retained only in this
        // scratch test: production callers never receive raw projections.
        let manifest = crate::workbench_sqlite_exporter::export_confirmed_db_to_json_dry_run(
            &fixture.config.db_path,
            &fixture.config.confirmed_db_path,
            &source_ref,
        )
        .expect("confirmed scratch DB export manifest");
        assert_eq!(manifest.mode, "dry_run");
        assert_eq!(manifest.status, "planned");
        assert_eq!(manifest.target_root_ref, source_ref);
        assert!(!manifest.export_hash.is_empty(), "canonical export hash required");
        let workflow_projection = manifest
            .projected_files
            .iter()
            .find(|file| file.path == "workflow-state.v0.json")
            .expect("workflow-state canonical projection");
        assert!(workflow_projection.canonical);
        assert!(!workflow_projection.projected_hash.is_empty());
        assert!(manifest
            .redaction_manifest
            .iter()
            .any(|entry| entry.contains("credential")));

        assert_eq!(
            fs::read(&fixture.state_path).expect("read sidecar after dry-run export"),
            sidecar_before,
            "export must retain the original sidecar byte-for-byte"
        );
        assert_eq!(
            fs::read(&fixture.config.db_path).expect("read DB after dry-run export"),
            database_before,
            "export must not mutate the DB-primary source"
        );

        // The M2 rollback boundary for this no-cutover scratch slice is a
        // no-op: preserve the source, restart the normal reconciliation, and
        // prove that neither retained source nor DB becomes a new writer.
        clear_storage_mode_cache_for_path_for_tests(&fixture.state_path);
        initialize_for_startup(&fixture.state_path).expect("no-op rollback restart readback");
        assert_eq!(
            fs::read(&fixture.state_path).expect("read sidecar after no-op rollback"),
            sidecar_before,
            "no-op rollback must retain the original sidecar"
        );
        assert_eq!(
            fs::read(&fixture.config.db_path).expect("read DB after no-op rollback"),
            database_before,
            "no-op rollback must not append a new DB mutation"
        );
    }

    #[test]
    fn m2_reference_slice_db_primary_denial_is_audited_and_never_replayed_as_success() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-denial");
        let before = m2_workflow_state_ledger_counts(&fixture.config);
        let json_before = fs::read(&fixture.state_path).expect("read pre-denial JSON projection");
        let expected_revision = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository")
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read authoritative M2 workflow revision");
        let request = crate::WorkItemStateUpdateRequest {
            project_root: fixture.project_root.clone(),
            work_item_id: fixture.work_item_id.clone(),
            next_state: "failed".to_string(),
            command_id: Some("m2-reference-slice-denied-command".to_string()),
            idempotency_key: Some("m2-reference-slice-denied-key".to_string()),
            expected_revision: Some(expected_revision),
        };

        let error = crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect_err("ready_to_dispatch -> failed must be denied by the M2 policy gate");
        assert!(
            error.contains("非法工作项状态跳转：ready_to_dispatch -> failed"),
            "{error}"
        );
        assert_eq!(
            fs::read(&fixture.state_path).expect("read denied JSON projection"),
            json_before,
            "denial must not change the business projection"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [
                before[0] + 1,
                before[1],
                before[2] + 1,
                before[3],
                before[4]
            ],
            "denial may append only its receipt and audit"
        );
        let connection =
            Connection::open_with_flags(&fixture.config.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open M2 fixture DB read only");
        let command_id = "m2-reference-slice-denied-command";
        let (receipt_status, audit_action): (String, String) = connection
            .query_row(
                "SELECT r.status, a.action
                 FROM command_receipts r
                 JOIN audit_records a ON a.command_id = r.command_id
                 WHERE r.command_id = ?1",
                [command_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("load denial receipt and audit");
        assert_eq!(receipt_status, "DENIED");
        assert_eq!(audit_action, "DENIED");

        let replay_error = crate::update_work_item_state_at(&fixture.state_path, &request)
            .expect_err("denied receipt must not be returned as a successful replay");
        assert!(
            replay_error.contains("m2_existing_receipt_not_successful")
                && replay_error.contains("status=DENIED"),
            "{replay_error}"
        );
        assert_eq!(
            fs::read(&fixture.state_path).expect("read denied replay JSON projection"),
            json_before,
            "denied replay must still leave the business projection untouched"
        );
        assert_eq!(
            m2_workflow_state_ledger_counts(&fixture.config),
            [
                before[0] + 1,
                before[1],
                before[2] + 1,
                before[3],
                before[4]
            ],
            "denied replay must not grow the M2 ledger"
        );
    }

    #[test]
    fn m2_reference_slice_command_identity_allows_new_commands_but_rejects_stale_revisions() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-reference-slice-command-identity");
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB-primary repository gate")
            .expect("DB-primary repository");
        let initial_revision = repository
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read initial M2 revision");
        let request = |command_id: &str, idempotency_key: &str, expected_revision: i64, next_state: &str| {
            crate::WorkItemStateUpdateRequest {
                project_root: fixture.project_root.clone(),
                work_item_id: fixture.work_item_id.clone(),
                next_state: next_state.to_string(),
                command_id: Some(command_id.to_string()),
                idempotency_key: Some(idempotency_key.to_string()),
                expected_revision: Some(expected_revision),
            }
        };

        let denied = request(
            "m2-reference-slice-denied-then-legal",
            "m2-reference-slice-denied-key",
            initial_revision,
            "failed",
        );
        let denial = crate::update_work_item_state_at(&fixture.state_path, &denied)
            .expect_err("the first command is denied without advancing M2 revision");
        assert!(
            denial.contains("非法工作项状态跳转：ready_to_dispatch -> failed"),
            "{denial}"
        );

        let legal = request(
            "m2-reference-slice-legal-after-denial",
            "m2-reference-slice-legal-key",
            initial_revision,
            "running",
        );
        let legal_result = crate::update_work_item_state_at(&fixture.state_path, &legal)
            .expect("a distinct legal command must not alias the denied receipt");
        let response_provenance = legal_result
            .snapshot
            .m2_port_provenance
            .as_ref()
            .expect("explicit M2 response must expose its versioned port provenance");
        assert_eq!(
            response_provenance.repository_port_version,
            crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION
        );
        assert_eq!(
            response_provenance.schema_version,
            crate::workbench_sqlite_schema_m2::M2_SCHEMA_VERSION
        );
        assert_eq!(response_provenance.caller_mode, "EXPLICIT_M2_REQUEST");
        let connection = Connection::open_with_flags(
            &fixture.config.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .expect("open DB provenance ledger");
        let (receipt_policy_ref, event_trace_context, audit_source_refs):
            (String, Option<String>, Option<String>) = connection
            .query_row(
                "SELECT r.policy_decision_ref, e.trace_context, a.source_refs
                 FROM command_receipts r
                 JOIN events e ON e.command_id = r.command_id
                 JOIN audit_records a ON a.command_id = r.command_id
                 WHERE r.command_id = ?1",
                ["m2-reference-slice-legal-after-denial"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("the committed M2 receipt, event, and audit must share provenance");
        let expected_trace = format!(
            "repository_port_version={};schema_version={};caller_mode=EXPLICIT_M2_REQUEST",
            crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
            crate::workbench_sqlite_schema_m2::M2_SCHEMA_VERSION,
        );
        assert_eq!(event_trace_context.as_deref(), Some(expected_trace.as_str()));
        assert!(receipt_policy_ref.starts_with("policy_gateway:allowed;"));
        assert!(receipt_policy_ref.ends_with(&expected_trace));
        assert!(audit_source_refs
            .as_deref()
            .is_some_and(|value| value.ends_with(&expected_trace)));
        drop(connection);
        let legal_receipt = legal_result.receipt_id.expect("legal receipt");

        let stale = request(
            "m2-reference-slice-stale-command",
            "m2-reference-slice-stale-key",
            initial_revision,
            "retry_pending",
        );
        let stale_error = crate::update_work_item_state_at(&fixture.state_path, &stale)
            .expect_err("a new command with the prior aggregate revision must fail closed");
        assert!(stale_error.contains("m2_workflow_state_expected_revision_stale"));

        let revision_after_running = repository
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read revision after running");
        assert!(revision_after_running > initial_revision);
        let retry_pending = request(
            "m2-reference-slice-state-cycle-retry",
            "m2-reference-slice-state-cycle-retry-key",
            revision_after_running,
            "retry_pending",
        );
        let retry_result = crate::update_work_item_state_at(&fixture.state_path, &retry_pending)
            .expect("running -> retry_pending is a distinct legal command");
        let retry_receipt = retry_result.receipt_id.expect("retry receipt");
        assert_ne!(legal_receipt, retry_receipt);

        let revision_after_retry = repository
            .m2_workflow_state_sidecar_revision(&fixture.workflow_id, &fixture.work_item_id)
            .expect("read revision after retry_pending");
        let running_again = request(
            "m2-reference-slice-state-cycle-running",
            "m2-reference-slice-state-cycle-running-key",
            revision_after_retry,
            "running",
        );
        let running_again_result = crate::update_work_item_state_at(&fixture.state_path, &running_again)
            .expect("retry_pending -> running must not collide with the first running command");
        assert_ne!(
            running_again_result.receipt_id.as_deref(),
            Some(legal_receipt.as_str())
        );

        let state = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read the projected reference slice");
        assert_eq!(state["work_items"][0]["state"], "running");
        let receipt_count: i64 = Connection::open_with_flags(
            &fixture.config.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .expect("open DB receipt ledger")
        .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| row.get(0))
        .expect("count command receipts");
        assert_eq!(
            receipt_count, 4,
            "one denied plus three logical commands; stale preflight must add no receipt"
        );
    }

    #[test]
    fn m5a_db_ahead_replays_on_restart_and_json_ahead_degrades_to_json() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("replay-and-block");
        let created = crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(&fixture),
            1_700_000_000_200,
            "m5a-replay-base",
        )
        .expect("base proposal");
        let mut db_only_proposal = serde_json::to_value(&created.proposal).expect("proposal JSON");
        db_only_proposal["proposal_id"] = Value::String("proposal:m5a:db-ahead".to_string());
        db_only_proposal["updated_at_ms"] = Value::from(1_700_000_000_201_i64);
        let mut db_only_audit = serde_json::to_value(&created.audit_event).expect("audit JSON");
        db_only_audit["audit_event_id"] = Value::String("audit:m5a:db-ahead".to_string());
        db_only_audit["proposal_id"] = Value::String("proposal:m5a:db-ahead".to_string());
        db_only_audit["created_at_ms"] = Value::from(1_700_000_000_201_i64);
        let repository = primary_repository_for_write(&fixture.state_path)
            .expect("DB primary repository")
            .expect("DB primary enabled");
        repository
            .record_proposal_with_audit(
                &db_only_proposal,
                &RepositoryAuditEntry {
                    event_id: "audit:m5a:db-ahead".to_string(),
                    target_kind: "project_consultation_proposal".to_string(),
                    target_id: "proposal:m5a:db-ahead".to_string(),
                    payload: db_only_audit,
                },
                None,
            )
            .expect("commit DB-only crash-window record");

        clear_storage_mode_cache_for_tests();
        initialize_for_startup(&fixture.state_path).expect("restart replays DB-leading projection");
        let proposal_store = crate::project_consultation_proposal_store::load_store(
            &fixture.state_path,
            crate::unix_timestamp_ms(),
        )
        .expect("read replayed proposal sidecar");
        assert!(proposal_store
            .proposals
            .iter()
            .any(|proposal| proposal.proposal_id == "proposal:m5a:db-ahead"));
        assert!(reconcile_db_vs_json(&fixture.config)
            .expect("reconcile replayed state")
            .is_green());

        let mut state = crate::read_workflow_state_value(&fixture.state_path)
            .expect("read state before JSON-leading injection");
        state["audit_events"]
            .as_array_mut()
            .expect("audit array")
            .push(json!({
                "event_id": "audit:m5a:json-leading",
                "event_type": "m5a_json_leading_injection",
                "target_ref": "m5a-json-leading",
                "actor_ref": "m5a-test",
                "source_kind": "workspace_state",
                "permission_level": "user_confirmed_write",
                "before_state": "none",
                "after_state": "injected",
                "created_at": "1700000000202",
                "reason": "test JSON-leading fail-closed"
            }));
        state["updated_at"] = Value::String("1700000000202".to_string());
        crate::write_validated_workflow_state(&fixture.state_path, &state)
            .expect("write JSON-leading test record");
        clear_storage_mode_cache_for_tests();
        let startup_error = initialize_for_startup(&fixture.state_path)
            .expect_err("JSON-leading state must block DB-primary startup");
        assert!(
            startup_error.contains("db_primary_projection_blocked"),
            "{startup_error}"
        );
        let db_before_fallback = db_primary_row_counts(&fixture.config);
        assert!(primary_repository_for_write(&fixture.state_path)
            .expect("blocked primary mode must degrade to JSON")
            .is_none());
        crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(&fixture),
            1_700_000_000_203,
            "m5a-json-ahead-json-fallback",
        )
        .expect("blocked product write must succeed through JSON fallback");
        let audits = degradation_audits(&fixture.state_path);
        assert_eq!(audits.len(), 1);
        assert!(
            audits[0]["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("db_json_reconciliation_not_green")),
            "{:?}",
            audits[0]
        );
        assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
        assert_db_primary_health_blocked(&fixture.state_path, "db_json_reconciliation_not_green");
    }

    #[test]
    fn m5a_projection_failure_blocks_db_writes_and_degrades_to_json() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("projection-failure-block");
        let error = complete_db_primary_json_projection(
            &fixture.state_path,
            "injected_projection_failure",
            || Err::<(), _>("injected JSON projection failure".to_string()),
        )
        .expect_err("a failed JSON projection must fail closed for this process");
        assert!(
            error.contains("injected JSON projection failure"),
            "{error}"
        );
        let db_before_fallback = db_primary_row_counts(&fixture.config);
        assert!(primary_repository_for_write(&fixture.state_path)
            .expect("subsequent DB-primary writes must degrade to JSON")
            .is_none());
        crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(&fixture),
            1_700_000_000_301,
            "m5a-projection-failure-json-fallback",
        )
        .expect("blocked product write must succeed through JSON fallback");
        let audits = degradation_audits(&fixture.state_path);
        assert_eq!(audits.len(), 1);
        assert!(
            audits[0]["reason"].as_str().is_some_and(|reason| {
                reason.contains("injected_projection_failure")
                    && reason.contains("injected JSON projection failure")
            }),
            "{:?}",
            audits[0]
        );
        assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
        assert_db_primary_health_blocked(&fixture.state_path, "injected_projection_failure");
        clear_storage_mode_cache_for_tests();
    }

    #[test]
    fn m2_t2_dispatch_refuses_a_blocked_db_primary_without_json_fallback() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("m2-t2-fail-closed-dispatch");
        let context = dispatch_context(&fixture);
        crate::write_prepared_dispatch(&fixture.state_path, context.clone())
            .expect("prepare the DB-primary dispatch before the injected gate failure");
        let json_before = fs::read(&fixture.state_path).expect("snapshot JSON before frozen write");
        let db_before = db_primary_row_counts(&fixture.config);
        let degradation_audits_before = degradation_audits(&fixture.state_path).len();
        block_db_primary_writes(
            &fixture.state_path,
            "m2_t2_post_commit_gate",
            "injected post-commit timeout",
        );

        let error = crate::write_started_dispatch(&fixture.state_path, &context)
            .expect_err("M2/T2 dispatch must freeze rather than fall back to JSON");
        assert!(
            error.contains("db_primary_m2_t2_write_frozen:workflow_node_dispatch_started")
                && error.contains("m2_t2_post_commit_gate"),
            "{error}"
        );
        assert_eq!(
            fs::read(&fixture.state_path).expect("read JSON after frozen write"),
            json_before,
            "a blocked M2/T2 mutation must not write a JSON projection, backup, or audit"
        );
        assert_eq!(
            db_primary_row_counts(&fixture.config),
            db_before,
            "a blocked M2/T2 mutation must not append DB records"
        );
        assert_eq!(
            degradation_audits(&fixture.state_path).len(),
            degradation_audits_before,
            "M2/T2 refusal must not record the historical JSON-fallback degradation audit"
        );
        clear_storage_mode_cache_for_tests();
    }

    #[test]
    fn m5a_blocked_mode_preserves_non_m2_legacy_fallback_flows_once_without_db_writes() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("blocked-six-flow-json-fallback");
        let supervisor_runtime = prepare_active_supervisor_run(&fixture);
        let db_before_fallback = db_primary_row_counts(&fixture.config);
        block_db_primary_writes(
            &fixture.state_path,
            "m5a_blocked_six_flow_fixture",
            "injected blocked fixture reason",
        );

        let proposal = crate::project_consultation_proposal_store::create_proposal(
            &fixture.state_path,
            &proposal_input(&fixture),
            1_700_000_000_500,
            "m5a-blocked-six-flow-proposal",
        )
        .expect("proposal flow must fall back to JSON");
        let _authorization = crate::plan_authorization_store::create_authorization(
            &fixture.state_path,
            &authorization_input(&fixture, &proposal.proposal.proposal_id),
            1_700_000_000_501,
            "m5a-blocked-six-flow-authorization",
        )
        .expect("authorization flow must fall back to JSON");
        let adapter = JsonFallbackSupervisorAdapter {
            executions: Cell::new(0),
        };
        let supervisor_result =
            crate::supervisor_action_controller::execute_supervisor_last_message(
                &supervisor_runtime,
                &json!({
                    "schema_version": "supervisor_action_proposal.v1",
                    "kind": "dispatch_worker",
                    "target": {
                        "node_id": format!("{}:node:codex-dev", fixture.workflow_id),
                        "work_item_id": fixture.work_item_id,
                    },
                    "reason": "exercise blocked JSON fallback",
                    "expected_result": "one JSON-only supervisor action"
                })
                .to_string(),
                &adapter,
            )
            .expect("supervisor action flow must fall back to JSON");

        assert_eq!(supervisor_result.status, "waiting_worker");
        assert_eq!(adapter.executions.get(), 1);
        let audits = degradation_audits(&fixture.state_path);
        assert_eq!(audits.len(), 1);
        assert!(
            audits[0]["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("m5a_blocked_six_flow_fixture")),
            "{:?}",
            audits[0]
        );
        assert_eq!(db_primary_row_counts(&fixture.config), db_before_fallback);
        assert_db_primary_health_blocked(&fixture.state_path, "m5a_blocked_six_flow_fixture");
        clear_storage_mode_cache_for_tests();
    }

    #[test]
    fn m5a_storage_mode_change_requires_restart() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("cached-mode");
        assert!(matches!(
            storage_mode_for(&fixture.state_path),
            StorageMode::DbPrimaryJsonProjection(_)
        ));

        let config_path = storage_mode_path(&fixture.state_path).expect("storage mode config path");
        fs::write(
            config_path,
            serde_json::to_vec(&json!({
                "schema_version": STORAGE_MODE_SCHEMA_VERSION,
                "mode": JSON_ONLY,
            }))
            .expect("serialize json-only mode"),
        )
        .expect("replace test config");
        assert!(matches!(
            storage_mode_for(&fixture.state_path),
            StorageMode::DbPrimaryJsonProjection(_)
        ));

        clear_storage_mode_cache_for_tests();
        assert!(matches!(
            storage_mode_for(&fixture.state_path),
            StorageMode::JsonOnly { .. }
        ));
    }

    #[test]
    fn m5a_reconciliation_is_read_only() {
        let _serial = test_lock().lock().expect("storage mode test lock");
        let fixture = db_primary_fixture("read-only-reconciliation");
        let state_before =
            fs::read(&fixture.state_path).expect("read workflow state before reconcile");
        let db_before = fs::read(&fixture.config.db_path).expect("read DB before reconcile");
        let proposal_sidecar =
            crate::project_consultation_proposal_store::sidecar_path(&fixture.state_path)
                .expect("proposal sidecar path");
        let authorization_sidecar =
            crate::plan_authorization_store::sidecar_path(&fixture.state_path)
                .expect("authorization sidecar path");
        let supervisor_sidecar = fixture
            .state_path
            .parent()
            .expect("workflow-state parent")
            .join("supervisor-action-control.v1.json");
        assert!(!proposal_sidecar.exists());
        assert!(!authorization_sidecar.exists());
        assert!(!supervisor_sidecar.exists());

        let report = reconcile_db_vs_json(&fixture.config).expect("read-only reconciliation");
        assert!(
            report.is_green(),
            "expected empty fixture to reconcile: {report:?}"
        );
        assert_eq!(
            fs::read(&fixture.state_path).expect("read workflow state after reconcile"),
            state_before
        );
        assert_eq!(
            fs::read(&fixture.config.db_path).expect("read DB after reconcile"),
            db_before
        );
        assert!(!proposal_sidecar.exists());
        assert!(!authorization_sidecar.exists());
        assert!(!supervisor_sidecar.exists());
    }
}
