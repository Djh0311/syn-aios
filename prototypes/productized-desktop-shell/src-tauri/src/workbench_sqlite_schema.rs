use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const WORKBENCH_SQLITE_SCHEMA_VERSION: &str = "workbench_sqlite_schema_v0";

pub(crate) const WORKBENCH_SQLITE_SCHEMA_DDL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS schema_migrations (
        version TEXT PRIMARY KEY,
        applied_at TEXT NOT NULL,
        description TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS import_batches (
        batch_id TEXT PRIMARY KEY,
        mode TEXT NOT NULL CHECK(mode IN ('dry_run', 'apply')),
        source_root_ref TEXT NOT NULL,
        source_root_hash TEXT NOT NULL,
        importer_version TEXT NOT NULL,
        started_at TEXT,
        finished_at TEXT,
        status TEXT NOT NULL CHECK(status IN ('accepted', 'applied', 'rejected', 'rolled_back')),
        dry_run_report_json TEXT
    )",
    "CREATE TABLE IF NOT EXISTS import_sources (
        source_id TEXT PRIMARY KEY,
        batch_id TEXT NOT NULL,
        source_kind TEXT NOT NULL,
        source_path_hash TEXT NOT NULL,
        source_hash TEXT,
        source_schema_version TEXT,
        detected_revision INTEGER,
        status TEXT NOT NULL CHECK(status IN ('accepted', 'missing_optional', 'rejected_corrupt', 'rejected_unknown', 'rejected_sensitive', 'skipped_duplicate')),
        warnings_json TEXT NOT NULL DEFAULT '[]',
        UNIQUE(batch_id, source_kind, source_path_hash, source_hash),
        FOREIGN KEY(batch_id) REFERENCES import_batches(batch_id)
    )",
    "CREATE TABLE IF NOT EXISTS source_records (
        source_record_id TEXT PRIMARY KEY,
        source_id TEXT NOT NULL,
        record_kind TEXT NOT NULL,
        natural_key TEXT NOT NULL,
        record_hash TEXT NOT NULL,
        status TEXT NOT NULL CHECK(status IN ('accepted', 'skipped_duplicate', 'conflict')),
        record_json TEXT NOT NULL,
        UNIQUE(source_id, record_kind, natural_key),
        FOREIGN KEY(source_id) REFERENCES import_sources(source_id)
    )",
    "CREATE TABLE IF NOT EXISTS export_batches (
        export_id TEXT PRIMARY KEY,
        source_batch_id TEXT,
        target_root_ref TEXT NOT NULL,
        export_hash TEXT NOT NULL,
        manifest_json TEXT NOT NULL,
        created_at TEXT,
        status TEXT NOT NULL CHECK(status IN ('planned', 'completed', 'failed', 'dry_run'))
    )",
    "CREATE TABLE IF NOT EXISTS rollback_points (
        rollback_id TEXT PRIMARY KEY,
        source_batch_id TEXT,
        source_backup_ref TEXT,
        db_checkpoint_ref TEXT,
        created_at TEXT,
        status TEXT NOT NULL,
        manifest_json TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS workflow_state_meta (
        workspace_id TEXT NOT NULL,
        source_root_hash TEXT NOT NULL,
        schema_version TEXT NOT NULL,
        workflow_version INTEGER NOT NULL,
        revision INTEGER,
        source_id TEXT,
        meta_json TEXT NOT NULL,
        PRIMARY KEY(workspace_id, source_root_hash)
    )",
    "CREATE TABLE IF NOT EXISTS projects (project_id TEXT PRIMARY KEY, source_id TEXT, project_root TEXT, path_hash TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS agent_adapters (adapter_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflows (workflow_id TEXT PRIMARY KEY, project_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_nodes (node_id TEXT PRIMARY KEY, workflow_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_edges (edge_id TEXT PRIMARY KEY, workflow_id TEXT, source_node_id TEXT, target_node_id TEXT, edge_type TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS work_items (work_item_id TEXT PRIMARY KEY, workflow_id TEXT, node_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_artifacts (artifact_id TEXT PRIMARY KEY, work_item_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_reviews (review_id TEXT PRIMARY KEY, workflow_id TEXT, work_item_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_audit_events (event_id TEXT PRIMARY KEY, target_kind TEXT, target_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_node_session_bindings (binding_id TEXT PRIMARY KEY, workflow_id TEXT, node_id TEXT, work_item_id TEXT, lifecycle TEXT, session_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_node_dispatches (dispatch_id TEXT PRIMARY KEY, workflow_id TEXT, node_id TEXT, work_item_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS capabilities (capability_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS harness_resources (resource_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_scopes (scope_id TEXT PRIMARY KEY, project_id TEXT, workflow_id TEXT, session_id TEXT, scope_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_source_refs (source_ref_id TEXT PRIMARY KEY, source_kind TEXT, source_id TEXT, source_hash TEXT, ref_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS formal_memory_records (memory_id TEXT PRIMARY KEY, scope_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS formal_memory_versions (version_id TEXT PRIMARY KEY, memory_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS formal_memory_audit_events (audit_event_id TEXT PRIMARY KEY, memory_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_candidates (candidate_key TEXT PRIMARY KEY, candidate_id TEXT UNIQUE, formal_memory_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_candidate_events (audit_ref_id TEXT PRIMARY KEY, candidate_key TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS observations (observation_key TEXT PRIMARY KEY, observation_id TEXT UNIQUE, candidate_key TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS observation_events (audit_ref_id TEXT PRIMARY KEY, observation_key TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_capture_events (event_key TEXT PRIMARY KEY, capture_event_id TEXT UNIQUE, observation_key TEXT, candidate_key TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_lint_runs (lint_run_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_lint_findings (finding_id TEXT PRIMARY KEY, lint_run_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS memory_entity_relations (relation_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS mature_pattern_candidates (candidate_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS mature_pattern_audit_events (audit_event_id TEXT PRIMARY KEY, candidate_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS blackboard_candidates (candidate_key TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS blackboard_candidate_audit_events (audit_event_id TEXT PRIMARY KEY, candidate_key TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS project_proposals (proposal_id TEXT PRIMARY KEY, project_id TEXT, workflow_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS project_proposal_decisions (decision_id TEXT PRIMARY KEY, proposal_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS project_proposal_audit_events (audit_event_id TEXT PRIMARY KEY, proposal_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS plan_authorizations (authorization_id TEXT PRIMARY KEY, source_proposal_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS plan_authorization_audit_events (audit_event_id TEXT PRIMARY KEY, authorization_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS authorized_execution_scopes (authorization_id TEXT PRIMARY KEY, scope_fingerprint TEXT, scope_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS stage_c_reviews (review_id TEXT PRIMARY KEY, review_kind TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS stage_c_acceptance_summaries (artifact_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS product_commands (product_command_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS product_command_previews (preview_id TEXT PRIMARY KEY, product_command_id TEXT, preview_hash TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS product_command_decisions (decision_id TEXT PRIMARY KEY, product_command_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS product_command_attempts (attempt_id TEXT PRIMARY KEY, product_command_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS session_continuations (continuation_id TEXT PRIMARY KEY, product_command_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS session_continuation_attempts (attempt_id TEXT PRIMARY KEY, continuation_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS session_continuation_audit_events (event_id TEXT PRIMARY KEY, continuation_id TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS runtime_log_entries (entry_id TEXT PRIMARY KEY, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS runtime_log_summaries (summary_id TEXT PRIMARY KEY, batch_id TEXT, category TEXT, status TEXT, severity TEXT, summary_hash TEXT, source_id TEXT, record_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS runtime_source_refs (runtime_source_ref_id TEXT PRIMARY KEY, entry_id TEXT, source_kind TEXT, source_ref_id TEXT, ref_json TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS readback_results (readback_id TEXT PRIMARY KEY, attempt_id TEXT, source_kind TEXT, source_id TEXT, record_hash TEXT NOT NULL, record_json TEXT NOT NULL)",
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_import_batches_source_mode ON import_batches(source_root_hash, importer_version, mode)",
    "CREATE INDEX IF NOT EXISTS idx_import_sources_batch_kind ON import_sources(batch_id, source_kind)",
    "CREATE INDEX IF NOT EXISTS idx_source_records_kind_natural ON source_records(record_kind, natural_key)",
    "CREATE INDEX IF NOT EXISTS idx_workflows_project ON workflows(project_id)",
    "CREATE INDEX IF NOT EXISTS idx_workflow_nodes_workflow ON workflow_nodes(workflow_id)",
    "CREATE INDEX IF NOT EXISTS idx_work_items_workflow_node ON work_items(workflow_id, node_id)",
    "CREATE INDEX IF NOT EXISTS idx_formal_memory_versions_memory ON formal_memory_versions(memory_id)",
    "CREATE INDEX IF NOT EXISTS idx_memory_candidates_formal_memory ON memory_candidates(formal_memory_id)",
    "CREATE INDEX IF NOT EXISTS idx_observations_candidate ON observations(candidate_key)",
    "CREATE INDEX IF NOT EXISTS idx_product_command_attempts_command ON product_command_attempts(product_command_id)",
    "CREATE INDEX IF NOT EXISTS idx_session_continuation_attempts_continuation ON session_continuation_attempts(continuation_id)",
    "CREATE INDEX IF NOT EXISTS idx_runtime_log_summaries_status ON runtime_log_summaries(category, status, severity)",
];

pub(crate) fn initialize_temp_workbench_sqlite_db(path: &Path) -> Result<(), String> {
    if !is_allowed_temp_or_fixture_path(path) {
        return Err(format!(
            "temp_or_fixture_path_required: refusing to initialize workbench sqlite outside temp or R3 fixture paths: {}",
            path.display()
        ));
    }
    initialize_workbench_sqlite_db(path, "temp")
}

pub(crate) fn initialize_confirmed_workbench_sqlite_db(
    path: &Path,
    confirmed_path: &Path,
) -> Result<(), String> {
    if path != confirmed_path {
        return Err(format!(
            "confirmed_db_path_mismatch: expected {} got {}",
            confirmed_path.display(),
            path.display()
        ));
    }
    if !path.is_absolute() {
        return Err(format!(
            "confirmed_db_path_absolute_required:{}",
            path.display()
        ));
    }
    initialize_workbench_sqlite_db(path, "confirmed")
}

fn initialize_workbench_sqlite_db(path: &Path, path_kind: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("sqlite db path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "create sqlite {path_kind} parent failed {}: {error}",
            parent.display()
        )
    })?;

    let connection = Connection::open(path).map_err(|error| {
        format!(
            "open {path_kind} workbench sqlite failed {}: {error}",
            path.display()
        )
    })?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("enable sqlite foreign keys failed: {error}"))?;
    connection
        .execute_batch(&WORKBENCH_SQLITE_SCHEMA_DDL.join(";\n"))
        .map_err(|error| format!("initialize workbench sqlite schema failed: {error}"))?;
    connection
        .execute(
            "INSERT OR IGNORE INTO schema_migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
            [
                WORKBENCH_SQLITE_SCHEMA_VERSION,
                "1970-01-01T00:00:00Z",
                "R3-A1 dry-run schema v0 initialization",
            ],
        )
        .map_err(|error| format!("record schema migration failed: {error}"))?;
    Ok(())
}

fn is_allowed_temp_or_fixture_path(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    let temp_dir = std::env::temp_dir();
    let fixture_dir = manifest_fixture_root();
    path.starts_with(&temp_dir) || path.starts_with(&fixture_dir)
}

fn manifest_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_schema_initializes_temp_db_with_core_tables() {
        let dir = temp_dir("sqlite-schema-core");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("r3-a1-workbench-test.sqlite");

        initialize_temp_workbench_sqlite_db(&db_path).expect("initialize temp sqlite db");

        let connection = Connection::open(&db_path).expect("open initialized db");
        for table in [
            "schema_migrations",
            "import_batches",
            "import_sources",
            "source_records",
            "workflow_state_meta",
            "projects",
            "formal_memory_records",
            "memory_candidates",
            "observations",
            "plan_authorizations",
            "project_proposals",
            "product_commands",
            "session_continuations",
            "runtime_log_entries",
        ] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("query table");
            assert_eq!(count, 1, "missing table {table}");
        }
    }

    #[test]
    fn sqlite_schema_rejects_non_temp_paths() {
        let err = initialize_temp_workbench_sqlite_db(Path::new("/var/workbench.sqlite"))
            .expect_err("non temp path must be rejected");
        assert!(err.contains("temp_or_fixture_path_required"));
    }

    #[test]
    fn sqlite_schema_hardens_core_constraints_and_indexes() {
        let dir = temp_dir("sqlite-schema-hardening");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("r3-a2-workbench-test.sqlite");

        initialize_temp_workbench_sqlite_db(&db_path).expect("initialize temp sqlite db");

        let connection = Connection::open(&db_path).expect("open initialized db");
        let import_batches_sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'import_batches'",
                [],
                |row| row.get(0),
            )
            .expect("query import_batches sql");
        assert!(import_batches_sql.contains("CHECK(mode IN ('dry_run', 'apply'))"));
        assert!(import_batches_sql.contains("CHECK(status IN"));

        for index in [
            "idx_import_batches_source_mode",
            "idx_source_records_kind_natural",
            "idx_workflow_nodes_workflow",
            "idx_formal_memory_versions_memory",
            "idx_product_command_attempts_command",
            "idx_runtime_log_summaries_status",
        ] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                    [index],
                    |row| row.get(0),
                )
                .expect("query index");
            assert_eq!(count, 1, "missing index {index}");
        }

        let invalid = connection.execute(
            "INSERT INTO import_batches (batch_id, mode, source_root_ref, source_root_hash, importer_version, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                "batch:invalid",
                "production",
                "fixture",
                "hash",
                "importer",
                "accepted",
            ],
        );
        assert!(invalid.is_err(), "invalid import mode should hit CHECK");
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("r3-a1-{prefix}-{nanos}"))
    }
}
