use crate::workbench_sqlite_apply::{apply_fixture_dir_to_temp_db, table_count};
use crate::workbench_sqlite_importer::canonical_json_hash;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "workbench_sqlite_transaction_acceptance.v1";
const MODE: &str = "level_a_fixture_transaction_acceptance";
const CANDIDATE_KEY: &str = "candidate:r3-a13:transaction";
const MEMORY_ID: &str = "memory:r3-a13:transaction";
const VERSION_ID: &str = "memory-version:r3-a13:transaction:1";
const MEMORY_AUDIT_ID: &str = "memory-audit:r3-a13:transaction:adopted";
const WORKFLOW_AUDIT_ID: &str = "audit:r3-a13:transaction:memory-adopted";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteTransactionAcceptanceFailurePoint {
    BeforeTransactionBegin,
    AfterCandidateUpdateBeforeFormalMemoryInsert,
    AfterFormalMemoryInsertBeforeVersionInsert,
    AfterVersionInsertBeforeMemoryAuditInsert,
    AfterMemoryAuditInsertBeforeWorkflowAuditInsert,
    BeforeCommit,
    AfterCommitBeforeReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteTransactionAcceptanceReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) status: String,
    pub(crate) db_path_ref: String,
    pub(crate) db_path_hash: String,
    pub(crate) candidate_key: String,
    pub(crate) memory_id: String,
    pub(crate) memory_version_id: String,
    pub(crate) memory_audit_event_id: String,
    pub(crate) workflow_audit_event_id: String,
    pub(crate) before_counts: BTreeMap<String, i64>,
    pub(crate) after_counts: BTreeMap<String, i64>,
    pub(crate) rows_changed: BTreeMap<String, usize>,
    pub(crate) failure_point: Option<String>,
    pub(crate) transaction_flags: SqliteTransactionAcceptanceFlags,
    pub(crate) rollback_assurance: SqliteTransactionRollbackAssurance,
    pub(crate) cutover_gap_matrix: Vec<SqliteCutoverGapItem>,
    pub(crate) do_not_claim: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteTransactionAcceptanceFlags {
    pub(crate) sqlite_transaction_used: bool,
    pub(crate) candidate_adopted: bool,
    pub(crate) formal_memory_created: bool,
    pub(crate) memory_version_created: bool,
    pub(crate) memory_audit_written: bool,
    pub(crate) workflow_audit_ref_written: bool,
    pub(crate) source_json_written: bool,
    pub(crate) sidecar_written: bool,
    pub(crate) production_db_written: bool,
    pub(crate) product_global_read_path_changed: bool,
    pub(crate) product_global_write_path_changed: bool,
    pub(crate) codex_home_touched: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteTransactionRollbackAssurance {
    pub(crate) status: String,
    pub(crate) before_commit_failure_leaves_no_half_adopted_state: bool,
    pub(crate) committed_rows_preserved_for_audit: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) instructions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteCutoverGapItem {
    pub(crate) item: String,
    pub(crate) level_a_status: String,
    pub(crate) level_b_status: String,
    pub(crate) acceptance: String,
}

pub(crate) fn rehearse_transaction_acceptance_level_a(
    fixture_root: &Path,
    db_path: &Path,
    failure_point: Option<SqliteTransactionAcceptanceFailurePoint>,
) -> Result<SqliteTransactionAcceptanceReport, String> {
    validate_fixture_root(fixture_root)?;
    validate_temp_db_path(db_path)?;

    if failure_point == Some(SqliteTransactionAcceptanceFailurePoint::BeforeTransactionBegin) {
        return Err("injected_failure_before_transaction_begin".to_string());
    }

    apply_fixture_dir_to_temp_db(fixture_root, db_path, None)?;
    let before_counts = tracked_counts(db_path)?;
    let mut connection = Connection::open(db_path)
        .map_err(|error| format!("open transaction acceptance db failed: {error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("enable sqlite foreign keys failed: {error}"))?;
    let transaction = connection
        .transaction()
        .map_err(|error| format!("begin transaction acceptance failed: {error}"))?;

    let candidate_rows = transaction
        .execute(
            "UPDATE memory_candidates
             SET formal_memory_id = ?1,
                 record_json = ?2,
                 record_hash = ?3
             WHERE candidate_key = ?4 AND formal_memory_id IS NULL",
            params![
                MEMORY_ID,
                json_string(&candidate_record_json())?,
                canonical_json_hash(&candidate_record_json()),
                CANDIDATE_KEY,
            ],
        )
        .map_err(|error| format!("update candidate adoption failed: {error}"))?;
    if candidate_rows != 1 {
        return Err("transaction_acceptance_candidate_not_pending".to_string());
    }
    inject_before_commit_failure(
        failure_point,
        SqliteTransactionAcceptanceFailurePoint::AfterCandidateUpdateBeforeFormalMemoryInsert,
    )?;

    let memory_rows = transaction
        .execute(
            "INSERT INTO formal_memory_records (memory_id, scope_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                MEMORY_ID,
                "scope:r3-a13:transaction",
                "r3-a13:transaction-acceptance",
                canonical_json_hash(&formal_memory_record_json()),
                json_string(&formal_memory_record_json())?,
            ],
        )
        .map_err(|error| format!("insert formal memory failed: {error}"))?;
    inject_before_commit_failure(
        failure_point,
        SqliteTransactionAcceptanceFailurePoint::AfterFormalMemoryInsertBeforeVersionInsert,
    )?;

    let version_rows = transaction
        .execute(
            "INSERT INTO formal_memory_versions (version_id, memory_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                VERSION_ID,
                MEMORY_ID,
                "r3-a13:transaction-acceptance",
                canonical_json_hash(&formal_memory_version_json()),
                json_string(&formal_memory_version_json())?,
            ],
        )
        .map_err(|error| format!("insert formal memory version failed: {error}"))?;
    inject_before_commit_failure(
        failure_point,
        SqliteTransactionAcceptanceFailurePoint::AfterVersionInsertBeforeMemoryAuditInsert,
    )?;

    let memory_audit_rows = transaction
        .execute(
            "INSERT INTO formal_memory_audit_events (audit_event_id, memory_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                MEMORY_AUDIT_ID,
                MEMORY_ID,
                "r3-a13:transaction-acceptance",
                canonical_json_hash(&memory_audit_json()),
                json_string(&memory_audit_json())?,
            ],
        )
        .map_err(|error| format!("insert formal memory audit failed: {error}"))?;
    inject_before_commit_failure(
        failure_point,
        SqliteTransactionAcceptanceFailurePoint::AfterMemoryAuditInsertBeforeWorkflowAuditInsert,
    )?;

    let workflow_audit_rows = transaction
        .execute(
            "INSERT INTO workflow_audit_events (event_id, target_kind, target_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                WORKFLOW_AUDIT_ID,
                "memory_candidate",
                CANDIDATE_KEY,
                "r3-a13:transaction-acceptance",
                canonical_json_hash(&workflow_audit_json()),
                json_string(&workflow_audit_json())?,
            ],
        )
        .map_err(|error| format!("insert workflow audit failed: {error}"))?;
    inject_before_commit_failure(
        failure_point,
        SqliteTransactionAcceptanceFailurePoint::BeforeCommit,
    )?;

    transaction
        .commit()
        .map_err(|error| format!("commit transaction acceptance failed: {error}"))?;

    let after_counts = tracked_counts(db_path)?;
    let mut rows_changed = BTreeMap::new();
    rows_changed.insert("memory_candidates".to_string(), candidate_rows);
    rows_changed.insert("formal_memory_records".to_string(), memory_rows);
    rows_changed.insert("formal_memory_versions".to_string(), version_rows);
    rows_changed.insert("formal_memory_audit_events".to_string(), memory_audit_rows);
    rows_changed.insert("workflow_audit_events".to_string(), workflow_audit_rows);

    let committed_but_report_failed =
        failure_point == Some(SqliteTransactionAcceptanceFailurePoint::AfterCommitBeforeReport);
    Ok(SqliteTransactionAcceptanceReport {
        schema_version: SCHEMA_VERSION.to_string(),
        mode: MODE.to_string(),
        status: if committed_but_report_failed {
            "committed_but_report_failed".to_string()
        } else {
            "completed".to_string()
        },
        db_path_ref: db_path.display().to_string(),
        db_path_hash: stable_hash(&db_path.display().to_string()),
        candidate_key: CANDIDATE_KEY.to_string(),
        memory_id: MEMORY_ID.to_string(),
        memory_version_id: VERSION_ID.to_string(),
        memory_audit_event_id: MEMORY_AUDIT_ID.to_string(),
        workflow_audit_event_id: WORKFLOW_AUDIT_ID.to_string(),
        before_counts,
        after_counts,
        rows_changed,
        failure_point: failure_point.map(|point| format!("{point:?}")),
        transaction_flags: completed_flags(),
        rollback_assurance: rollback_assurance(committed_but_report_failed),
        cutover_gap_matrix: cutover_gap_matrix(),
        do_not_claim: do_not_claim(),
    })
}

fn inject_before_commit_failure(
    actual: Option<SqliteTransactionAcceptanceFailurePoint>,
    expected: SqliteTransactionAcceptanceFailurePoint,
) -> Result<(), String> {
    if actual == Some(expected) {
        Err(format!("injected_failure_{expected:?}"))
    } else {
        Ok(())
    }
}

fn tracked_counts(db_path: &Path) -> Result<BTreeMap<String, i64>, String> {
    let mut counts = BTreeMap::new();
    for table in [
        "memory_candidates",
        "formal_memory_records",
        "formal_memory_versions",
        "formal_memory_audit_events",
        "workflow_audit_events",
    ] {
        counts.insert(table.to_string(), table_count(db_path, table)?);
    }
    Ok(counts)
}

fn validate_fixture_root(path: &Path) -> Result<(), String> {
    if path.is_absolute() && path.starts_with(manifest_r3_a13_fixture_root()) && path.is_dir() {
        Ok(())
    } else {
        Err(format!(
            "r3_a13_fixture_root_required: refusing transaction acceptance fixture outside R3-A13 fixtures: {}",
            path.display()
        ))
    }
}

fn validate_temp_db_path(path: &Path) -> Result<(), String> {
    if path.is_absolute() && path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_db_path_required: refusing transaction acceptance db outside temp: {}",
            path.display()
        ))
    }
}

fn manifest_r3_a13_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a13")
}

fn candidate_record_json() -> Value {
    json!({
        "candidate_key": CANDIDATE_KEY,
        "candidate_id": "candidate-id:r3-a13:transaction",
        "status": "adopted",
        "formal_memory_id": MEMORY_ID,
        "adoption_mode": "sqlite_transaction_acceptance_level_a"
    })
}

fn formal_memory_record_json() -> Value {
    json!({
        "memory_id": MEMORY_ID,
        "scope_id": "scope:r3-a13:transaction",
        "status": "active",
        "claim": "R3-A13 proves candidate adoption, formal memory, version, memory audit, and workflow audit can commit in one SQLite transaction."
    })
}

fn formal_memory_version_json() -> Value {
    json!({
        "version_id": VERSION_ID,
        "memory_id": MEMORY_ID,
        "status": "current",
        "source_candidate_key": CANDIDATE_KEY
    })
}

fn memory_audit_json() -> Value {
    json!({
        "audit_event_id": MEMORY_AUDIT_ID,
        "memory_id": MEMORY_ID,
        "event_kind": "candidate_adopted_to_formal_memory",
        "candidate_key": CANDIDATE_KEY,
        "workflow_audit_event_id": WORKFLOW_AUDIT_ID
    })
}

fn workflow_audit_json() -> Value {
    json!({
        "event_id": WORKFLOW_AUDIT_ID,
        "target_kind": "memory_candidate",
        "target_id": CANDIDATE_KEY,
        "event_kind": "sqlite_transaction_acceptance",
        "memory_id": MEMORY_ID,
        "memory_audit_event_id": MEMORY_AUDIT_ID
    })
}

fn json_string(value: &Value) -> Result<String, String> {
    serde_json::to_string(value)
        .map_err(|error| format!("serialize acceptance json failed: {error}"))
}

fn completed_flags() -> SqliteTransactionAcceptanceFlags {
    SqliteTransactionAcceptanceFlags {
        sqlite_transaction_used: true,
        candidate_adopted: true,
        formal_memory_created: true,
        memory_version_created: true,
        memory_audit_written: true,
        workflow_audit_ref_written: true,
        source_json_written: false,
        sidecar_written: false,
        production_db_written: false,
        product_global_read_path_changed: false,
        product_global_write_path_changed: false,
        codex_home_touched: false,
    }
}

fn rollback_assurance(committed_but_report_failed: bool) -> SqliteTransactionRollbackAssurance {
    SqliteTransactionRollbackAssurance {
        status: if committed_but_report_failed {
            "committed_rows_preserved_for_audit".to_string()
        } else {
            "before_commit_failures_roll_back_atomically".to_string()
        },
        before_commit_failure_leaves_no_half_adopted_state: true,
        committed_rows_preserved_for_audit: committed_but_report_failed,
        production_restore_performed: false,
        instructions: vec![
            "do_not_restore_or_mutate_production_json_from_this_level_a_rehearsal".to_string(),
            "rerun_from_fixture_after_classifying_failure".to_string(),
            "preserve_temp_db_for_audit_if_commit_succeeded_before_report_failure".to_string(),
        ],
    }
}

fn cutover_gap_matrix() -> Vec<SqliteCutoverGapItem> {
    vec![
        gap(
            "R3-A9 production DB apply",
            "complete",
            "pending",
            "level_a_only",
        ),
        gap(
            "R3-A10 limited read-cut",
            "complete",
            "pending",
            "level_a_only",
        ),
        gap(
            "R3-A11 production observation",
            "complete",
            "pending",
            "level_a_only",
        ),
        gap(
            "R3-A12 stop-write decision",
            "complete",
            "pending",
            "level_a_only",
        ),
        gap(
            "R3-A13 transaction acceptance",
            "complete",
            "not_requested",
            "level_a_transaction_verified",
        ),
        gap(
            "production DB apply",
            "not_applicable",
            "pending",
            "deferred",
        ),
        gap(
            "production read-cut",
            "not_applicable",
            "pending",
            "deferred",
        ),
        gap(
            "production observation",
            "not_applicable",
            "pending",
            "deferred",
        ),
        gap(
            "JSON / sidecar stop-write",
            "not_applicable",
            "pending",
            "deferred",
        ),
        gap(
            "app startup / Tauri command / UI product path cutover",
            "not_applicable",
            "pending",
            "deferred",
        ),
        gap(
            "multi-agent parallel real execution unlock",
            "not_applicable",
            "pending",
            "blocked_until_real_cutover",
        ),
    ]
}

fn gap(
    item: &str,
    level_a_status: &str,
    level_b_status: &str,
    acceptance: &str,
) -> SqliteCutoverGapItem {
    SqliteCutoverGapItem {
        item: item.to_string(),
        level_a_status: level_a_status.to_string(),
        level_b_status: level_b_status.to_string(),
        acceptance: acceptance.to_string(),
    }
}

fn do_not_claim() -> Vec<String> {
    vec![
        "r3_full_completion".to_string(),
        "production_sqlite_migration_complete".to_string(),
        "production_db_created".to_string(),
        "production_read_cut_complete".to_string(),
        "json_sidecar_stop_write_complete".to_string(),
        "product_path_cut_to_db".to_string(),
        "multi_agent_parallel_real_execution_unlocked".to_string(),
    ]
}

fn stable_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_transaction_acceptance_commits_memory_and_audit_in_one_transaction() {
        let db_path = temp_db("success");
        let report = rehearse_transaction_acceptance_level_a(&fixture_dir(), &db_path, None)
            .expect("report");

        assert_eq!(report.status, "completed");
        assert!(report.transaction_flags.sqlite_transaction_used);
        assert!(report.transaction_flags.candidate_adopted);
        assert!(report.transaction_flags.formal_memory_created);
        assert!(report.transaction_flags.memory_version_created);
        assert!(report.transaction_flags.memory_audit_written);
        assert!(report.transaction_flags.workflow_audit_ref_written);
        assert!(!report.transaction_flags.source_json_written);
        assert!(!report.transaction_flags.production_db_written);
        assert_eq!(
            table_count(&db_path, "formal_memory_records").expect("formal memory"),
            1
        );
        assert_eq!(
            table_count(&db_path, "formal_memory_versions").expect("memory versions"),
            1
        );
        assert_eq!(
            table_count(&db_path, "formal_memory_audit_events").expect("memory audit"),
            1
        );
        assert_eq!(
            table_count(&db_path, "workflow_audit_events").expect("workflow audit"),
            2
        );
        assert!(report.cutover_gap_matrix.iter().any(|item| item.item
            == "JSON / sidecar stop-write"
            && item.level_b_status == "pending"));
    }

    #[test]
    fn sqlite_transaction_acceptance_rolls_back_all_before_commit_failures() {
        for point in [
            SqliteTransactionAcceptanceFailurePoint::AfterCandidateUpdateBeforeFormalMemoryInsert,
            SqliteTransactionAcceptanceFailurePoint::AfterFormalMemoryInsertBeforeVersionInsert,
            SqliteTransactionAcceptanceFailurePoint::AfterVersionInsertBeforeMemoryAuditInsert,
            SqliteTransactionAcceptanceFailurePoint::AfterMemoryAuditInsertBeforeWorkflowAuditInsert,
            SqliteTransactionAcceptanceFailurePoint::BeforeCommit,
        ] {
            let db_path = temp_db(&format!("rollback-{point:?}"));
            let err = rehearse_transaction_acceptance_level_a(
                &fixture_dir(),
                &db_path,
                Some(point),
            )
            .expect_err("before commit injection should fail");
            assert!(err.contains("injected_failure"));
            assert_eq!(
                table_count(&db_path, "formal_memory_records").expect("formal memory"),
                0,
                "{point:?}"
            );
            assert_eq!(
                table_count(&db_path, "formal_memory_versions").expect("memory versions"),
                0,
                "{point:?}"
            );
            assert_eq!(
                table_count(&db_path, "formal_memory_audit_events").expect("memory audit"),
                0,
                "{point:?}"
            );
            assert_eq!(
                table_count(&db_path, "workflow_audit_events").expect("workflow audit"),
                1,
                "{point:?}"
            );
            assert_candidate_still_pending(&db_path);
        }
    }

    #[test]
    fn sqlite_transaction_acceptance_classifies_after_commit_report_failure() {
        let db_path = temp_db("after-commit");
        let report = rehearse_transaction_acceptance_level_a(
            &fixture_dir(),
            &db_path,
            Some(SqliteTransactionAcceptanceFailurePoint::AfterCommitBeforeReport),
        )
        .expect("after commit report failure should preserve classified report");

        assert_eq!(report.status, "committed_but_report_failed");
        assert_eq!(
            report.rollback_assurance.status,
            "committed_rows_preserved_for_audit"
        );
        assert_eq!(
            table_count(&db_path, "formal_memory_records").expect("formal memory"),
            1
        );
    }

    #[test]
    fn sqlite_transaction_acceptance_rejects_non_temp_db_and_non_a13_fixture() {
        let err = rehearse_transaction_acceptance_level_a(
            &fixture_dir(),
            Path::new("/var/r3-a13.sqlite"),
            None,
        )
        .expect_err("non-temp db should reject");
        assert!(err.contains("temp_db_path_required"));

        let dir = std::env::temp_dir().join("r3-a13-non-fixture");
        fs::create_dir_all(&dir).expect("create temp fixture");
        let err = rehearse_transaction_acceptance_level_a(&dir, &temp_db("bad-fixture"), None)
            .expect_err("non fixture root should reject");
        assert!(err.contains("r3_a13_fixture_root_required"));
    }

    #[test]
    fn sqlite_transaction_acceptance_rejects_before_begin_without_creating_db() {
        let db_path = temp_db("before-begin");
        let err = rehearse_transaction_acceptance_level_a(
            &fixture_dir(),
            &db_path,
            Some(SqliteTransactionAcceptanceFailurePoint::BeforeTransactionBegin),
        )
        .expect_err("before begin should fail before db create");
        assert!(err.contains("injected_failure_before_transaction_begin"));
        assert!(!db_path.exists());
    }

    fn assert_candidate_still_pending(db_path: &Path) {
        let connection = Connection::open(db_path).expect("open db");
        let formal_memory_id: Option<String> = connection
            .query_row(
                "SELECT formal_memory_id FROM memory_candidates WHERE candidate_key = ?1",
                [CANDIDATE_KEY],
                |row| row.get(0),
            )
            .expect("query candidate");
        assert!(formal_memory_id.is_none());
    }

    fn fixture_dir() -> PathBuf {
        manifest_r3_a13_fixture_root().join("transaction-acceptance-core")
    }

    fn temp_db(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("r3-a13-{name}-{nanos}.sqlite"))
    }
}
