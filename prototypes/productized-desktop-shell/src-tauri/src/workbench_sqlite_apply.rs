use crate::workbench_sqlite_importer::{
    canonical_json_hash, dry_run_import_fixture_dir, CANONICAL_RUNTIME_LOG,
    LEGACY_RUNTIME_LOG_ALIAS, OPTIONAL_SIDECARS, PRIMARY_WORKFLOW_STATE,
};
use crate::workbench_sqlite_schema::initialize_temp_workbench_sqlite_db;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) const WORKBENCH_SQLITE_APPLY_IMPORTER_VERSION: &str =
    "workbench_sqlite_apply_importer_v0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteApplyFailurePoint {
    BeforeDbBegin,
    AfterDbBeginBeforeFirstInsert,
    AfterImportBatchBeforeDomainInsert,
    AfterFirstDomainInsertBeforeCommit,
    BeforeCommit,
    AfterCommitBeforeExportManifest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteApplyImportReport {
    pub(crate) batch_id: String,
    pub(crate) status: String,
    pub(crate) source_root_ref: String,
    pub(crate) source_root_hash: String,
    pub(crate) records_inserted: usize,
    pub(crate) records_skipped: usize,
    pub(crate) sources_inserted: usize,
    pub(crate) failure_point: Option<String>,
    pub(crate) runtime_log_alias_policy: String,
}

pub(crate) fn apply_fixture_dir_to_temp_db(
    fixture_root: &Path,
    db_path: &Path,
    failure_point: Option<SqliteApplyFailurePoint>,
) -> Result<SqliteApplyImportReport, String> {
    if !is_allowed_temp_or_r3_fixture_db_path(db_path) {
        return Err(format!(
            "temp_or_fixture_path_required: refusing to apply workbench sqlite outside temp or R3 fixture paths: {}",
            db_path.display()
        ));
    }
    if failure_point == Some(SqliteApplyFailurePoint::BeforeDbBegin) {
        return Err("injected_failure_before_db_begin".to_string());
    }

    let dry_run = dry_run_import_fixture_dir(fixture_root)?;
    if dry_run.batch_status != "accepted" && dry_run.batch_status != "accepted_with_rejections" {
        return Err(format!(
            "dry_run_batch_not_applyable:{}",
            dry_run.batch_status
        ));
    }
    if dry_run.counts.conflicts > 0
        || dry_run
            .source_inventory
            .iter()
            .any(|source| source.classification == "rejected_sensitive")
    {
        return Err(format!(
            "dry_run_batch_not_applyable:{}",
            dry_run.batch_status
        ));
    }

    initialize_temp_workbench_sqlite_db(db_path)?;
    let mut connection = Connection::open(db_path)
        .map_err(|error| format!("open temp workbench sqlite apply db failed: {error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("enable sqlite foreign keys failed: {error}"))?;

    let transaction = connection
        .transaction()
        .map_err(|error| format!("begin sqlite apply transaction failed: {error}"))?;
    if failure_point == Some(SqliteApplyFailurePoint::AfterDbBeginBeforeFirstInsert) {
        return Err("injected_failure_after_db_begin_before_first_insert".to_string());
    }

    let batch_id = format!("r3-a2-apply:{}", dry_run.source_root_hash);
    let dry_run_report_json = serde_json::to_string(&dry_run)
        .map_err(|error| format!("serialize dry-run report for apply failed: {error}"))?;
    transaction
        .execute(
            "INSERT INTO import_batches (batch_id, mode, source_root_ref, source_root_hash, importer_version, started_at, finished_at, status, dry_run_report_json)
             VALUES (?1, 'apply', ?2, ?3, ?4, ?5, ?5, 'applied', ?6)
             ON CONFLICT(batch_id) DO NOTHING",
            params![
                batch_id,
                dry_run.source_root_ref,
                dry_run.source_root_hash,
                WORKBENCH_SQLITE_APPLY_IMPORTER_VERSION,
                "1970-01-01T00:00:00Z",
                dry_run_report_json,
            ],
        )
        .map_err(|error| format!("insert import batch failed: {error}"))?;

    let mut source_ids = BTreeMap::new();
    let mut sources_inserted = 0usize;
    for source in dry_run
        .source_inventory
        .iter()
        .filter(|source| source.classification == "accepted")
    {
        let source_id = format!("source:{}:{}", source.source_kind, source.source_path_hash);
        let warnings_json = serde_json::to_string(&source.warnings)
            .map_err(|error| format!("serialize source warnings failed: {error}"))?;
        let inserted = transaction
            .execute(
                "INSERT INTO import_sources (source_id, batch_id, source_kind, source_path_hash, source_hash, source_schema_version, detected_revision, status, warnings_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(source_id) DO NOTHING",
                params![
                    source_id,
                    batch_id,
                    source.source_kind,
                    source.source_path_hash,
                    source.source_hash,
                    source.source_schema_version,
                    source.detected_revision,
                    source.classification,
                    warnings_json,
                ],
            )
            .map_err(|error| format!("insert import source failed: {error}"))?;
        sources_inserted += inserted;
        source_ids.insert(source.source_kind.clone(), source_id);
    }

    if failure_point == Some(SqliteApplyFailurePoint::AfterImportBatchBeforeDomainInsert) {
        return Err("injected_failure_after_import_batch_before_domain_insert".to_string());
    }

    let mut records_inserted = 0usize;
    let mut records_skipped = 0usize;
    let mut first_domain_insert_seen = false;
    let fixture_files = load_fixture_values(fixture_root)?;
    for (source_name, value) in fixture_files {
        let source_kind = source_kind_for_file(&source_name);
        if source_kind == "runtime_log_legacy_alias" {
            continue;
        }
        let Some(source_id) = source_ids.get(source_kind).cloned() else {
            continue;
        };
        for record in records_for_source(&source_name, &value) {
            let record_json = serde_json::to_string(&record.value)
                .map_err(|error| format!("serialize record failed: {error}"))?;
            let source_record_id = format!(
                "source-record:{source_kind}:{}",
                stable_sqlite_key(&record.record_kind, &record.natural_key)
            );
            let source_record_inserted = transaction
                .execute(
                    "INSERT INTO source_records (source_record_id, source_id, record_kind, natural_key, record_hash, status, record_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'accepted', ?6)
                     ON CONFLICT(source_record_id) DO NOTHING",
                    params![
                        source_record_id,
                        source_id,
                        record.record_kind,
                        record.natural_key,
                        record.record_hash,
                        record_json,
                    ],
                )
                .map_err(|error| format!("insert source record failed: {error}"))?;
            records_skipped += usize::from(source_record_inserted == 0);
            let domain_inserted = insert_domain_record(
                &transaction,
                source_kind,
                &source_id,
                &record.record_kind,
                &record.natural_key,
                &record.record_hash,
                &record.value,
            )?;
            if domain_inserted {
                records_inserted += 1;
                if !first_domain_insert_seen {
                    first_domain_insert_seen = true;
                    if failure_point
                        == Some(SqliteApplyFailurePoint::AfterFirstDomainInsertBeforeCommit)
                    {
                        return Err(
                            "injected_failure_after_first_domain_insert_before_commit".to_string()
                        );
                    }
                }
            } else {
                records_skipped += 1;
            }
        }
    }

    if failure_point == Some(SqliteApplyFailurePoint::BeforeCommit) {
        return Err("injected_failure_before_commit".to_string());
    }
    transaction
        .commit()
        .map_err(|error| format!("commit sqlite apply transaction failed: {error}"))?;

    let report = SqliteApplyImportReport {
        batch_id,
        status: "applied".to_string(),
        source_root_ref: dry_run.source_root_ref,
        source_root_hash: dry_run.source_root_hash,
        records_inserted,
        records_skipped,
        sources_inserted,
        failure_point: failure_point.map(|point| format!("{point:?}")),
        runtime_log_alias_policy: dry_run.runtime_log_alias.policy,
    };
    if failure_point == Some(SqliteApplyFailurePoint::AfterCommitBeforeExportManifest) {
        return Err("injected_failure_after_commit_before_export_manifest".to_string());
    }
    Ok(report)
}

fn is_allowed_temp_or_r3_fixture_db_path(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    path.starts_with(std::env::temp_dir())
        || path.starts_with(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join("r3-a2"),
        )
}

#[derive(Clone)]
struct FixtureRecord {
    record_kind: String,
    natural_key: String,
    record_hash: String,
    value: Value,
}

fn load_fixture_values(root: &Path) -> Result<Vec<(String, Value)>, String> {
    let mut names = vec![PRIMARY_WORKFLOW_STATE.to_string()];
    names.extend(OPTIONAL_SIDECARS.iter().map(|name| (*name).to_string()));
    let mut values = Vec::new();
    for name in names {
        let path = root.join(&name);
        if !path.exists() {
            continue;
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("read fixture source failed {name}: {error}"))?;
        let value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("parse fixture source failed {name}: {error}"))?;
        values.push((name, value));
    }
    Ok(values)
}

fn records_for_source(source_name: &str, value: &Value) -> Vec<FixtureRecord> {
    match source_name {
        PRIMARY_WORKFLOW_STATE => workflow_records(value),
        "formal-memories.v1.json" => sidecar_records(
            value,
            &[
                ("records", "formal_memory_record", &["memory_id", "id"][..]),
                (
                    "versions",
                    "formal_memory_version",
                    &["version_id", "id"][..],
                ),
                (
                    "audit_events",
                    "formal_memory_audit_event",
                    &["audit_event_id", "event_id", "id"][..],
                ),
            ],
        ),
        "memory-candidates.v1.json" => sidecar_records(
            value,
            &[
                (
                    "candidates",
                    "memory_candidate",
                    &["candidate_key", "candidate_id", "id"][..],
                ),
                (
                    "events",
                    "memory_candidate_event",
                    &["audit_ref_id", "event_id", "id"][..],
                ),
            ],
        ),
        "observations.v1.json" => sidecar_records(
            value,
            &[
                (
                    "observations",
                    "observation",
                    &["observation_key", "observation_id", "id"][..],
                ),
                (
                    "events",
                    "observation_event",
                    &["audit_ref_id", "event_id", "id"][..],
                ),
            ],
        ),
        "memory-capture-events.v1.json" => sidecar_records(
            value,
            &[(
                "events",
                "memory_capture_event",
                &["event_key", "capture_event_id", "id"][..],
            )],
        ),
        "plan-authorizations.v1.json" => sidecar_records(
            value,
            &[
                (
                    "authorizations",
                    "plan_authorization",
                    &["authorization_id", "id"][..],
                ),
                (
                    "audit_events",
                    "plan_authorization_audit_event",
                    &["audit_event_id", "event_id", "id"][..],
                ),
            ],
        ),
        "project-proposals.v1.json" => sidecar_records(
            value,
            &[
                ("proposals", "project_proposal", &["proposal_id", "id"][..]),
                (
                    "decisions",
                    "project_proposal_decision",
                    &["decision_id", "id"][..],
                ),
                (
                    "audit_events",
                    "project_proposal_audit_event",
                    &["audit_event_id", "event_id", "id"][..],
                ),
            ],
        ),
        "real-execution-product-commands.v1.json" => sidecar_records(
            value,
            &[
                (
                    "commands",
                    "product_command",
                    &["product_command_id", "command_id", "id"][..],
                ),
                (
                    "previews",
                    "product_command_preview",
                    &["preview_id", "id"][..],
                ),
                (
                    "decisions",
                    "product_command_decision",
                    &["decision_id", "id"][..],
                ),
                (
                    "attempts",
                    "product_command_attempt",
                    &["attempt_id", "id"][..],
                ),
            ],
        ),
        "session-continuations.v1.json" => sidecar_records(
            value,
            &[
                (
                    "continuations",
                    "session_continuation",
                    &["continuation_id", "id"][..],
                ),
                (
                    "attempts",
                    "session_continuation_attempt",
                    &["attempt_id", "id"][..],
                ),
                (
                    "audit_events",
                    "session_continuation_audit_event",
                    &["event_id", "audit_event_id", "id"][..],
                ),
            ],
        ),
        CANONICAL_RUNTIME_LOG => sidecar_records(
            value,
            &[
                ("entries", "runtime_log_entry", &["entry_id", "id"][..]),
                (
                    "summaries",
                    "runtime_log_summary",
                    &["summary_id", "id"][..],
                ),
            ],
        ),
        _ => Vec::new(),
    }
}

fn workflow_records(value: &Value) -> Vec<FixtureRecord> {
    let specs: &[(&str, &str, &[&str])] = &[
        ("projects", "projects", &["project_id", "id"]),
        ("agent_adapters", "agent_adapters", &["adapter_id", "id"]),
        ("workflows", "workflows", &["workflow_id", "id"]),
        ("nodes", "nodes", &["node_id", "id"]),
        ("edges", "edges", &["edge_id", "id"]),
        ("work_items", "work_items", &["work_item_id", "id"]),
        ("artifacts", "artifacts", &["artifact_id", "id"]),
        ("reviews", "reviews", &["review_id", "id"]),
        (
            "audit_events",
            "audit_events",
            &["event_id", "audit_event_id", "id"],
        ),
        ("capabilities", "capabilities", &["capability_id", "id"]),
        (
            "harness_resources",
            "harness_resources",
            &["resource_id", "id"],
        ),
        (
            "workflow_node_session_bindings",
            "workflow_node_session_bindings",
            &["binding_id", "session_id", "id"],
        ),
        (
            "workflow_node_dispatches",
            "workflow_node_dispatches",
            &["dispatch_id", "id"],
        ),
    ];
    let mut records = sidecar_records(value, specs);
    let meta = serde_json::json!({
        "schema_version": value.get("schema_version").cloned().unwrap_or(Value::Null),
        "workflow_version": value.get("workflow_version").cloned().unwrap_or(Value::Null),
        "revision": value.get("revision").cloned().unwrap_or(Value::Null)
    });
    records.push(record_from_value(
        "workflow_state_meta",
        "workflow_state_meta",
        meta,
    ));
    records
}

fn sidecar_records(value: &Value, specs: &[(&str, &str, &[&str])]) -> Vec<FixtureRecord> {
    let mut records = Vec::new();
    for (array, record_kind, keys) in specs {
        if let Some(items) = value.get(*array).and_then(Value::as_array) {
            for (index, item) in items.iter().enumerate() {
                let natural_key = natural_key(item, keys).unwrap_or_else(|| {
                    format!("{record_kind}:{index}:{}", canonical_json_hash(item))
                });
                records.push(record_from_value(record_kind, &natural_key, item.clone()));
            }
        }
    }
    records
}

fn record_from_value(record_kind: &str, natural_key: &str, value: Value) -> FixtureRecord {
    FixtureRecord {
        record_kind: record_kind.to_string(),
        natural_key: natural_key.to_string(),
        record_hash: canonical_json_hash(&value),
        value,
    }
}

fn natural_key(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn insert_domain_record(
    transaction: &rusqlite::Transaction<'_>,
    source_kind: &str,
    source_id: &str,
    record_kind: &str,
    natural_key: &str,
    record_hash: &str,
    value: &Value,
) -> Result<bool, String> {
    let record_json = serde_json::to_string(value)
        .map_err(|error| format!("serialize domain record failed: {error}"))?;
    let inserted = match record_kind {
        "workflow_state_meta" => transaction.execute(
            "INSERT INTO workflow_state_meta (workspace_id, source_root_hash, schema_version, workflow_version, revision, source_id, meta_json)
             VALUES ('fixture-workspace', ?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(workspace_id, source_root_hash) DO NOTHING",
            params![
                source_id,
                value.get("schema_version").and_then(Value::as_str).unwrap_or("workflow_state_v0"),
                value.get("workflow_version").and_then(Value::as_i64).unwrap_or(1),
                value.get("revision").and_then(Value::as_i64),
                source_id,
                record_json,
            ],
        ),
        "projects" => transaction.execute(
            "INSERT INTO projects (project_id, source_id, project_root, path_hash, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(project_id) DO NOTHING",
            params![
                natural_key,
                source_id,
                value.get("project_root").and_then(Value::as_str),
                value.get("project_root").and_then(Value::as_str).map(stable_hash),
                record_hash,
                record_json,
            ],
        ),
        "agent_adapters" => insert_simple(
            transaction,
            "agent_adapters",
            "adapter_id",
            natural_key,
            source_id,
            record_hash,
            &record_json,
        ),
        "workflows" => transaction.execute(
            "INSERT INTO workflows (workflow_id, project_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(workflow_id) DO NOTHING",
            params![
                natural_key,
                value.get("project_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "nodes" => transaction.execute(
            "INSERT INTO workflow_nodes (node_id, workflow_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(node_id) DO NOTHING",
            params![
                natural_key,
                value.get("workflow_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "edges" => transaction.execute(
            "INSERT INTO workflow_edges (edge_id, workflow_id, source_node_id, target_node_id, edge_type, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) ON CONFLICT(edge_id) DO NOTHING",
            params![
                natural_key,
                value.get("workflow_id").and_then(Value::as_str),
                value.get("source_node_id").and_then(Value::as_str),
                value.get("target_node_id").and_then(Value::as_str),
                value.get("edge_type").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "work_items" => transaction.execute(
            "INSERT INTO work_items (work_item_id, workflow_id, node_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(work_item_id) DO NOTHING",
            params![
                natural_key,
                value.get("workflow_id").and_then(Value::as_str),
                value.get("node_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "artifacts" => transaction.execute(
            "INSERT INTO workflow_artifacts (artifact_id, work_item_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(artifact_id) DO NOTHING",
            params![
                natural_key,
                value.get("work_item_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "reviews" => transaction.execute(
            "INSERT INTO workflow_reviews (review_id, workflow_id, work_item_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(review_id) DO NOTHING",
            params![
                natural_key,
                value.get("workflow_id").and_then(Value::as_str),
                value.get("work_item_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "audit_events" => transaction.execute(
            "INSERT INTO workflow_audit_events (event_id, target_kind, target_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(event_id) DO NOTHING",
            params![
                natural_key,
                value.get("target_kind").and_then(Value::as_str),
                value.get("target_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "workflow_node_session_bindings" => transaction.execute(
            "INSERT INTO workflow_node_session_bindings (binding_id, workflow_id, node_id, work_item_id, lifecycle, session_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) ON CONFLICT(binding_id) DO NOTHING",
            params![
                natural_key,
                value.get("workflow_id").and_then(Value::as_str),
                value.get("node_id").and_then(Value::as_str),
                value.get("work_item_id").and_then(Value::as_str),
                value.get("lifecycle").and_then(Value::as_str),
                value.get("session_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "workflow_node_dispatches" => transaction.execute(
            "INSERT INTO workflow_node_dispatches (dispatch_id, workflow_id, node_id, work_item_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT(dispatch_id) DO NOTHING",
            params![
                natural_key,
                value.get("workflow_id").and_then(Value::as_str),
                value.get("node_id").and_then(Value::as_str),
                value.get("work_item_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "capabilities" => insert_simple(
            transaction,
            "capabilities",
            "capability_id",
            natural_key,
            source_id,
            record_hash,
            &record_json,
        ),
        "harness_resources" => insert_simple(
            transaction,
            "harness_resources",
            "resource_id",
            natural_key,
            source_id,
            record_hash,
            &record_json,
        ),
        "formal_memory_record" => transaction.execute(
            "INSERT INTO formal_memory_records (memory_id, scope_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(memory_id) DO NOTHING",
            params![natural_key, value.get("scope_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "formal_memory_version" => transaction.execute(
            "INSERT INTO formal_memory_versions (version_id, memory_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(version_id) DO NOTHING",
            params![natural_key, value.get("memory_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "formal_memory_audit_event" => transaction.execute(
            "INSERT INTO formal_memory_audit_events (audit_event_id, memory_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(audit_event_id) DO NOTHING",
            params![natural_key, value.get("memory_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "memory_candidate" => transaction.execute(
            "INSERT INTO memory_candidates (candidate_key, candidate_id, formal_memory_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(candidate_key) DO NOTHING",
            params![
                natural_key,
                value.get("candidate_id").and_then(Value::as_str),
                value.get("formal_memory_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "memory_candidate_event" => transaction.execute(
            "INSERT INTO memory_candidate_events (audit_ref_id, candidate_key, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(audit_ref_id) DO NOTHING",
            params![natural_key, value.get("candidate_key").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "observation" => transaction.execute(
            "INSERT INTO observations (observation_key, observation_id, candidate_key, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(observation_key) DO NOTHING",
            params![
                natural_key,
                value.get("observation_id").and_then(Value::as_str),
                value.get("candidate_key").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "observation_event" => transaction.execute(
            "INSERT INTO observation_events (audit_ref_id, observation_key, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(audit_ref_id) DO NOTHING",
            params![natural_key, value.get("observation_key").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "memory_capture_event" => transaction.execute(
            "INSERT INTO memory_capture_events (event_key, capture_event_id, observation_key, candidate_key, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT(event_key) DO NOTHING",
            params![
                natural_key,
                value.get("capture_event_id").and_then(Value::as_str),
                value.get("observation_key").and_then(Value::as_str),
                value.get("candidate_key").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "project_proposal" => transaction.execute(
            "INSERT INTO project_proposals (proposal_id, project_id, workflow_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(proposal_id) DO NOTHING",
            params![
                natural_key,
                value.get("project_id").and_then(Value::as_str),
                value.get("workflow_id").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "project_proposal_decision" => transaction.execute(
            "INSERT INTO project_proposal_decisions (decision_id, proposal_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(decision_id) DO NOTHING",
            params![natural_key, value.get("proposal_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "project_proposal_audit_event" => transaction.execute(
            "INSERT INTO project_proposal_audit_events (audit_event_id, proposal_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(audit_event_id) DO NOTHING",
            params![natural_key, value.get("proposal_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "plan_authorization" => transaction.execute(
            "INSERT INTO plan_authorizations (authorization_id, source_proposal_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(authorization_id) DO NOTHING",
            params![natural_key, value.get("source_proposal_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "plan_authorization_audit_event" => transaction.execute(
            "INSERT INTO plan_authorization_audit_events (audit_event_id, authorization_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(audit_event_id) DO NOTHING",
            params![natural_key, value.get("authorization_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "product_command" => insert_simple(
            transaction,
            "product_commands",
            "product_command_id",
            natural_key,
            source_id,
            record_hash,
            &record_json,
        ),
        "product_command_preview" => transaction.execute(
            "INSERT INTO product_command_previews (preview_id, product_command_id, preview_hash, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(preview_id) DO NOTHING",
            params![
                natural_key,
                value.get("product_command_id").and_then(Value::as_str),
                value.get("preview_hash").and_then(Value::as_str),
                source_id,
                record_hash,
                record_json,
            ],
        ),
        "product_command_decision" => transaction.execute(
            "INSERT INTO product_command_decisions (decision_id, product_command_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(decision_id) DO NOTHING",
            params![natural_key, value.get("product_command_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "product_command_attempt" => transaction.execute(
            "INSERT INTO product_command_attempts (attempt_id, product_command_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(attempt_id) DO NOTHING",
            params![natural_key, value.get("product_command_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "session_continuation" => transaction.execute(
            "INSERT INTO session_continuations (continuation_id, product_command_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(continuation_id) DO NOTHING",
            params![natural_key, value.get("product_command_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "session_continuation_attempt" => transaction.execute(
            "INSERT INTO session_continuation_attempts (attempt_id, continuation_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(attempt_id) DO NOTHING",
            params![natural_key, value.get("continuation_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "session_continuation_audit_event" => transaction.execute(
            "INSERT INTO session_continuation_audit_events (event_id, continuation_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(event_id) DO NOTHING",
            params![natural_key, value.get("continuation_id").and_then(Value::as_str), source_id, record_hash, record_json],
        ),
        "runtime_log_entry" => insert_simple(
            transaction,
            "runtime_log_entries",
            "entry_id",
            natural_key,
            source_id,
            record_hash,
            &record_json,
        ),
        "runtime_log_summary" => transaction.execute(
            "INSERT INTO runtime_log_summaries (summary_id, batch_id, category, status, severity, summary_hash, source_id, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) ON CONFLICT(summary_id) DO NOTHING",
            params![
                natural_key,
                value.get("batch_id").and_then(Value::as_str),
                value.get("category").and_then(Value::as_str),
                value.get("status").and_then(Value::as_str),
                value.get("severity").and_then(Value::as_str),
                record_hash,
                source_id,
                record_json,
            ],
        ),
        _ => Ok(0),
    }
    .map_err(|error| format!("insert domain record {record_kind}:{natural_key} failed: {error}"))?;
    let _ = source_kind;
    Ok(inserted > 0)
}

fn insert_simple(
    transaction: &rusqlite::Transaction<'_>,
    table: &str,
    id_column: &str,
    id: &str,
    source_id: &str,
    record_hash: &str,
    record_json: &str,
) -> rusqlite::Result<usize> {
    let sql = format!(
        "INSERT INTO {table} ({id_column}, source_id, record_hash, record_json) VALUES (?1, ?2, ?3, ?4) ON CONFLICT({id_column}) DO NOTHING"
    );
    transaction.execute(&sql, params![id, source_id, record_hash, record_json])
}

fn source_kind_for_file(name: &str) -> &'static str {
    match name {
        PRIMARY_WORKFLOW_STATE => "workflow_state",
        "formal-memories.v1.json" => "formal_memory",
        "memory-candidates.v1.json" => "memory_candidate",
        "memory-capture-events.v1.json" => "memory_capture",
        "observations.v1.json" => "observation",
        "plan-authorizations.v1.json" => "plan_authorization",
        "project-proposals.v1.json" => "project_proposal",
        "real-execution-product-commands.v1.json" => "product_command",
        "session-continuations.v1.json" => "session_continuation",
        CANONICAL_RUNTIME_LOG => "runtime_log",
        LEGACY_RUNTIME_LOG_ALIAS => "runtime_log_legacy_alias",
        _ => "unknown_sidecar",
    }
}

fn stable_sqlite_key(record_kind: &str, natural_key: &str) -> String {
    stable_hash(&format!("{record_kind}:{natural_key}"))
}

fn stable_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn table_count(db_path: &Path, table: &str) -> Result<i64, String> {
    let connection = Connection::open(db_path)
        .map_err(|error| format!("open sqlite count db failed {}: {error}", db_path.display()))?;
    let sql = format!("SELECT COUNT(*) FROM {table}");
    connection
        .query_row(&sql, [], |row| row.get(0))
        .map_err(|error| format!("count table {table} failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_apply_importer_applies_valid_chain_and_reapply_is_idempotent() {
        let fixture = fixture_dir("apply-valid-core-chain");
        let db_path = temp_db("apply-valid-core-chain");

        let first = apply_fixture_dir_to_temp_db(&fixture, &db_path, None).expect("first apply");
        let second = apply_fixture_dir_to_temp_db(&fixture, &db_path, None).expect("second apply");

        assert_eq!(first.status, "applied");
        assert!(first.records_inserted >= 8);
        assert_eq!(second.records_inserted, 0);
        assert!(second.records_skipped >= first.records_inserted);
        assert_eq!(table_count(&db_path, "projects").expect("projects"), 1);
        assert_eq!(
            table_count(&db_path, "product_commands").expect("product commands"),
            1
        );
        assert_eq!(
            table_count(&db_path, "runtime_log_entries").expect("runtime entries"),
            1
        );
    }

    #[test]
    fn sqlite_apply_importer_rejects_conflicts_sensitive_and_corrupt_without_partial_rows() {
        for name in [
            "apply-conflict-rollback",
            "apply-revision-conflict-rollback",
            "apply-corrupt-primary-reject",
            "apply-sensitive-reject",
        ] {
            let db_path = temp_db(name);
            let err = apply_fixture_dir_to_temp_db(&fixture_dir(name), &db_path, None)
                .expect_err("fixture should reject");
            assert!(err.contains("dry_run_batch_not_applyable") || err.contains("rejected"));
            if db_path.exists() {
                assert_eq!(
                    table_count(&db_path, "projects").expect("projects"),
                    0,
                    "{name}"
                );
                assert_eq!(
                    table_count(&db_path, "product_commands").expect("product commands"),
                    0,
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn sqlite_apply_importer_rolls_back_failure_injection_before_commit() {
        for point in [
            SqliteApplyFailurePoint::AfterDbBeginBeforeFirstInsert,
            SqliteApplyFailurePoint::AfterImportBatchBeforeDomainInsert,
            SqliteApplyFailurePoint::AfterFirstDomainInsertBeforeCommit,
            SqliteApplyFailurePoint::BeforeCommit,
        ] {
            let db_path = temp_db(&format!("failure-{point:?}"));
            let err = apply_fixture_dir_to_temp_db(
                &fixture_dir("crash-after-domain-before-commit"),
                &db_path,
                Some(point),
            )
            .expect_err("failure injection should fail before commit");
            assert!(err.contains("injected_failure"));
            if db_path.exists() {
                assert_eq!(table_count(&db_path, "projects").expect("projects"), 0);
                assert_eq!(
                    table_count(&db_path, "import_batches").expect("import batches"),
                    0
                );
            }
        }
    }

    #[test]
    fn sqlite_apply_importer_rejects_before_db_begin_without_creating_db() {
        let db_path = temp_db("failure-before-db-begin");
        let err = apply_fixture_dir_to_temp_db(
            &fixture_dir("apply-valid-core-chain"),
            &db_path,
            Some(SqliteApplyFailurePoint::BeforeDbBegin),
        )
        .expect_err("before begin injection should fail before opening sqlite");

        assert!(err.contains("injected_failure_before_db_begin"));
        assert!(
            !db_path.exists(),
            "before begin injection must not create temp db"
        );
    }

    #[test]
    fn sqlite_apply_importer_after_commit_injection_keeps_committed_rows() {
        let db_path = temp_db("failure-after-commit");
        let err = apply_fixture_dir_to_temp_db(
            &fixture_dir("crash-after-source-before-domain"),
            &db_path,
            Some(SqliteApplyFailurePoint::AfterCommitBeforeExportManifest),
        )
        .expect_err("after commit injection should return failure");
        assert!(err.contains("injected_failure_after_commit_before_export_manifest"));
        assert_eq!(table_count(&db_path, "projects").expect("projects"), 1);
        assert_eq!(
            table_count(&db_path, "import_batches").expect("import batches"),
            1
        );
    }

    #[test]
    fn sqlite_apply_importer_rejects_non_temp_db_path() {
        let err = apply_fixture_dir_to_temp_db(
            &fixture_dir("apply-valid-core-chain"),
            Path::new("/var/r3-a2.sqlite"),
            None,
        )
        .expect_err("non-temp db should reject");
        assert!(err.contains("temp_or_fixture_path_required"));
    }

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("r3-a2")
            .join(name)
    }

    fn temp_db(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("r3-a2-{name}-{nanos}.sqlite"))
    }
}
