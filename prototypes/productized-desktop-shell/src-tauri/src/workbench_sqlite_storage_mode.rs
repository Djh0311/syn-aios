use crate::utils::hash::sha256_hex;
use crate::workbench_sqlite_repository::{
    ConfirmedWorkbenchSqliteRepositoryConfig, RepositoryAuditEntry, WorkbenchSqliteRepository,
    CONFIRMED_DB_DENIED_PATH_MARKERS,
};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
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
    primary_repository_for_write, workflow_state_write_route, WorkflowStateWriteRoute,
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
                let report = reconcile_db_vs_json(&config)?;
                if report.has_json_leading_or_divergence() {
                    return Err(report.fail_closed_reason());
                }
                let replayed_db_primary_projection = report.has_db_leading();
                if replayed_db_primary_projection {
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

fn append_startup_mode_audit(
    config: &DbPrimaryJsonProjectionConfig,
    replayed_db_primary_projection: bool,
) -> Result<(), String> {
    let timestamp = crate::unix_timestamp_string();
    let mut value = crate::read_workflow_state_value(&config.workflow_state_path)?;
    let event_id = crate::workflow_audit::audit_event_identity(
        "storage-mode-startup",
        &config.db_path_hash(),
        &timestamp,
    );
    let event = json!({
        "event_id": event_id,
        "event_type": if replayed_db_primary_projection { "storage_mode_projection_replayed" } else { "storage_mode_initialized" },
        "target_ref": config.db_path_hash(),
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
    use std::sync::Mutex;

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
    fn m5a_blocked_mode_degrades_all_six_product_flows_to_json_once_without_db_writes() {
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
        let authorization = crate::plan_authorization_store::create_authorization(
            &fixture.state_path,
            &authorization_input(&fixture, &proposal.proposal.proposal_id),
            1_700_000_000_501,
            "m5a-blocked-six-flow-authorization",
        )
        .expect("authorization flow must fall back to JSON");
        let mut context = dispatch_context(&fixture);
        context.binding_id = "binding:m5a:blocked-six-flow".to_string();
        context.native_thread_id = "thread:m5a:blocked-six-flow".to_string();
        context.plan_authorization_id = Some(authorization.authorization.authorization_id.clone());
        crate::write_prepared_dispatch(&fixture.state_path, context.clone())
            .expect("prepared dispatch flow must fall back to JSON");
        crate::write_started_dispatch(&fixture.state_path, &context)
            .expect("started dispatch flow must fall back to JSON");
        crate::update_work_item_state_at(
            &fixture.state_path,
            &crate::WorkItemStateUpdateRequest {
                project_root: fixture.project_root.clone(),
                work_item_id: fixture.work_item_id.clone(),
                next_state: "ready_for_review".to_string(),
            },
        )
        .expect("work item flow must fall back to JSON");
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
