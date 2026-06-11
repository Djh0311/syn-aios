use crate::workbench_sqlite_apply::{apply_fixture_dir_to_temp_db, table_count};
use crate::workbench_sqlite_exporter::{export_temp_db_to_json_dry_run, SqliteProjectedFile};
use crate::workbench_sqlite_importer::canonical_json_hash;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteObservationFailurePoint {
    BeforeObservationSample,
    AfterFirstExportBeforeSecondSample,
    ExportHashMismatch,
    ProjectionFileMissing,
    ProjectionFileCorrupt,
    RollbackManifestMissing,
    RollbackManifestIncomplete,
    DbIntegrityOrSchemaMismatch,
    ObservationDriftBetweenSamples,
    AfterRollbackSelectedBeforeReportCommit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteObservationRehearsalReport {
    pub(crate) observation_status: String,
    pub(crate) stable_verified: bool,
    pub(crate) degraded: bool,
    pub(crate) source_root_hash: String,
    pub(crate) db_path_ref: String,
    pub(crate) projection_root_ref: String,
    pub(crate) rollback_manifest_path_ref: String,
    pub(crate) observation_report_path_ref: String,
    pub(crate) sample_one: SqliteObservationSample,
    pub(crate) sample_two: SqliteObservationSample,
    pub(crate) export_verification: SqliteExportVerification,
    pub(crate) rollback_recovery_verification: SqliteObservationRollbackVerification,
    pub(crate) failure_point: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteObservationSample {
    pub(crate) sample_label: String,
    pub(crate) export_hash: String,
    pub(crate) projection_hash: String,
    pub(crate) projected_files: Vec<SqliteObservationProjectedFile>,
    pub(crate) db_row_counts: BTreeMap<String, i64>,
    pub(crate) redaction_policy: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteObservationProjectedFile {
    pub(crate) path: String,
    pub(crate) hash: String,
    pub(crate) record_count: usize,
    pub(crate) redaction_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteExportVerification {
    pub(crate) status: String,
    pub(crate) source_root_hash: String,
    pub(crate) db_export_hash: String,
    pub(crate) projection_hash: String,
    pub(crate) export_manifest_hash: String,
    pub(crate) runtime_log_alias_policy: String,
    pub(crate) projected_files: Vec<SqliteObservationProjectedFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteObservationRollbackVerification {
    pub(crate) status: String,
    pub(crate) would_disable_db_read_cut: bool,
    pub(crate) would_use_last_verified_json_projection: bool,
    pub(crate) would_preserve_db_for_audit: bool,
    pub(crate) would_require_supervisor_decision: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) selected_projection_hash: String,
    pub(crate) instructions: Vec<String>,
}

pub(crate) fn rehearse_fixture_observation_period(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    observation_report_path: &Path,
    rollback_manifest_path: &Path,
    failure_point: Option<SqliteObservationFailurePoint>,
) -> Result<SqliteObservationRehearsalReport, String> {
    validate_fixture_root(fixture_root)?;
    validate_temp_db_path(db_path)?;
    validate_projection_paths(
        projection_root,
        observation_report_path,
        rollback_manifest_path,
    )?;
    remove_file_if_exists(observation_report_path)?;

    if failure_point == Some(SqliteObservationFailurePoint::BeforeObservationSample) {
        return Err("injected_failure_before_observation_sample".to_string());
    }

    let apply_report = apply_fixture_dir_to_temp_db(fixture_root, db_path, None)?;
    let source_root_hash = apply_report.source_root_hash.clone();

    if failure_point == Some(SqliteObservationFailurePoint::DbIntegrityOrSchemaMismatch) {
        fs::write(db_path, b"not a sqlite database").map_err(|error| {
            format!(
                "write corrupt observation db fixture failed {}: {error}",
                db_path.display()
            )
        })?;
    }

    let sample_one = observation_sample(db_path, projection_root, "sample_1")?;
    if sample_one.export_hash.is_empty() {
        return Err("observation_blocked:empty_export_hash".to_string());
    }

    if failure_point == Some(SqliteObservationFailurePoint::AfterFirstExportBeforeSecondSample) {
        remove_file_if_exists(observation_report_path)?;
        return Err("injected_failure_after_first_export_before_second_sample".to_string());
    }

    let mut sample_two = observation_sample(db_path, projection_root, "sample_2")?;
    if failure_point == Some(SqliteObservationFailurePoint::ObservationDriftBetweenSamples) {
        sample_two.projection_hash = "injected_observation_drift".to_string();
    }

    verify_sample_stability(&sample_one, &sample_two)?;
    let mut export_verification = export_verification(&source_root_hash, &sample_two);
    if failure_point == Some(SqliteObservationFailurePoint::ExportHashMismatch) {
        export_verification.db_export_hash = "injected_export_hash_mismatch".to_string();
    }
    verify_export_hashes(&export_verification, &sample_two)?;

    write_projection_files(db_path, projection_root)?;
    if failure_point == Some(SqliteObservationFailurePoint::ProjectionFileMissing) {
        if let Some(file) = sample_two.projected_files.first() {
            remove_file_if_exists(&projection_root.join(&file.path))?;
        }
    }
    if failure_point == Some(SqliteObservationFailurePoint::ProjectionFileCorrupt) {
        if let Some(file) = sample_two.projected_files.first() {
            fs::write(projection_root.join(&file.path), b"{corrupt").map_err(|error| {
                format!(
                    "write corrupt observation projection fixture failed {}: {error}",
                    file.path
                )
            })?;
        }
    }
    verify_projection_files(projection_root, &sample_two.projected_files)?;

    let rollback_verification = rollback_verification(&sample_two.projection_hash);
    let rollback_manifest = rollback_manifest_value(
        fixture_root,
        db_path,
        projection_root,
        rollback_manifest_path,
        &source_root_hash,
        &sample_two.projected_files,
        &sample_two.db_row_counts,
        &sample_two.redaction_policy,
        &rollback_verification,
    );
    commit_rollback_manifest(rollback_manifest_path, &rollback_manifest)?;
    verify_rollback_manifest(rollback_manifest_path)?;
    if failure_point == Some(SqliteObservationFailurePoint::RollbackManifestMissing) {
        remove_file_if_exists(rollback_manifest_path)?;
    }
    if failure_point == Some(SqliteObservationFailurePoint::RollbackManifestIncomplete) {
        write_json_file(
            rollback_manifest_path,
            &json!({
                "schema_version": "workbench_sqlite_observation_period.v1",
                "status": "manifest_commit_failed_before_complete",
                "failure_point": "RollbackManifestIncomplete"
            }),
        )?;
    }
    verify_rollback_manifest(rollback_manifest_path)?;

    if failure_point == Some(SqliteObservationFailurePoint::AfterRollbackSelectedBeforeReportCommit)
    {
        remove_file_if_exists(observation_report_path)?;
        return Err("injected_failure_after_rollback_selected_before_report_commit".to_string());
    }

    let report = SqliteObservationRehearsalReport {
        observation_status: "stable_verified".to_string(),
        stable_verified: true,
        degraded: false,
        source_root_hash,
        db_path_ref: db_path.display().to_string(),
        projection_root_ref: projection_root.display().to_string(),
        rollback_manifest_path_ref: rollback_manifest_path.display().to_string(),
        observation_report_path_ref: observation_report_path.display().to_string(),
        sample_one,
        sample_two,
        export_verification,
        rollback_recovery_verification: rollback_verification,
        failure_point: failure_point.map(|point| format!("{point:?}")),
    };
    write_observation_report(observation_report_path, &report)?;
    Ok(report)
}

fn observation_sample(
    db_path: &Path,
    projection_root: &Path,
    label: &str,
) -> Result<SqliteObservationSample, String> {
    verify_db_integrity(db_path)?;
    let manifest = export_temp_db_to_json_dry_run(db_path, &projection_root.display().to_string())?;
    verify_runtime_log_alias_policy(&manifest.projected_files)?;
    let projected_files = observation_projected_files(&manifest.projected_files);
    let projection_hash = projection_hash(&projected_files);
    Ok(SqliteObservationSample {
        sample_label: label.to_string(),
        export_hash: manifest.export_hash,
        projection_hash,
        projected_files,
        db_row_counts: db_row_counts(db_path)?,
        redaction_policy: manifest.redaction_manifest,
    })
}

fn verify_sample_stability(
    first: &SqliteObservationSample,
    second: &SqliteObservationSample,
) -> Result<(), String> {
    if first.export_hash != second.export_hash {
        return Err(format!(
            "observation_blocked:export_hash_drift:first={}:second={}",
            first.export_hash, second.export_hash
        ));
    }
    if first.projection_hash != second.projection_hash {
        return Err(format!(
            "observation_blocked:projection_hash_drift:first={}:second={}",
            first.projection_hash, second.projection_hash
        ));
    }
    if first.projected_files != second.projected_files {
        return Err("observation_blocked:projected_file_drift".to_string());
    }
    if first.db_row_counts != second.db_row_counts {
        return Err("observation_blocked:db_row_count_drift".to_string());
    }
    if first.redaction_policy != second.redaction_policy {
        return Err("observation_blocked:redaction_policy_drift".to_string());
    }
    Ok(())
}

fn verify_export_hashes(
    verification: &SqliteExportVerification,
    sample: &SqliteObservationSample,
) -> Result<(), String> {
    if verification.db_export_hash != sample.export_hash {
        return Err(format!(
            "observation_blocked:export_hash_mismatch:expected={}:actual={}",
            sample.export_hash, verification.db_export_hash
        ));
    }
    if verification.projection_hash != sample.projection_hash {
        return Err(format!(
            "observation_blocked:projection_hash_mismatch:expected={}:actual={}",
            sample.projection_hash, verification.projection_hash
        ));
    }
    Ok(())
}

fn export_verification(
    source_root_hash: &str,
    sample: &SqliteObservationSample,
) -> SqliteExportVerification {
    SqliteExportVerification {
        status: "verified".to_string(),
        source_root_hash: source_root_hash.to_string(),
        db_export_hash: sample.export_hash.clone(),
        projection_hash: sample.projection_hash.clone(),
        export_manifest_hash: canonical_json_hash(&json!({
            "source_root_hash": source_root_hash,
            "export_hash": sample.export_hash,
            "projection_hash": sample.projection_hash,
            "projected_files": sample.projected_files,
            "redaction_policy": sample.redaction_policy
        })),
        runtime_log_alias_policy:
            "canonical_runtime_logs_only:runtime-logs.v1.json emitted; runtime-log.v1.json omitted"
                .to_string(),
        projected_files: sample.projected_files.clone(),
    }
}

fn observation_projected_files(
    files: &[SqliteProjectedFile],
) -> Vec<SqliteObservationProjectedFile> {
    files
        .iter()
        .map(|file| SqliteObservationProjectedFile {
            path: file.path.clone(),
            hash: file.projected_hash.clone(),
            record_count: file.record_count,
            redaction_status: "forbidden_sensitive_fields_omitted".to_string(),
        })
        .collect()
}

fn projection_hash(files: &[SqliteObservationProjectedFile]) -> String {
    canonical_json_hash(&json!({ "projected_files": files }))
}

fn write_projection_files(db_path: &Path, projection_root: &Path) -> Result<(), String> {
    let manifest = export_temp_db_to_json_dry_run(db_path, &projection_root.display().to_string())?;
    fs::create_dir_all(projection_root).map_err(|error| {
        format!(
            "create sqlite observation projection root failed {}: {error}",
            projection_root.display()
        )
    })?;
    remove_legacy_runtime_log_alias(projection_root)?;
    for file in manifest.projected_files {
        if file.path == "runtime-log.v1.json" {
            return Err("observation_blocked:legacy_runtime_log_alias_exported".to_string());
        }
        write_projected_file(projection_root, &file)?;
    }
    Ok(())
}

fn write_projected_file(root: &Path, file: &SqliteProjectedFile) -> Result<(), String> {
    if file.path.contains('/') || file.path.contains('\\') {
        return Err(format!(
            "observation_blocked:projection_file_name_must_be_flat:{}",
            file.path
        ));
    }
    write_json_file(&root.join(&file.path), &file.projection)
}

fn verify_projection_files(
    projection_root: &Path,
    files: &[SqliteObservationProjectedFile],
) -> Result<(), String> {
    for file in files {
        if file.path == "runtime-log.v1.json" {
            return Err("observation_blocked:legacy_runtime_log_alias_present".to_string());
        }
        let path = projection_root.join(&file.path);
        let bytes = fs::read(&path).map_err(|error| {
            format!(
                "observation_blocked:projection_file_missing:{}:{error}",
                file.path
            )
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "observation_blocked:projection_file_corrupt:{}:{error}",
                file.path
            )
        })?;
        let actual = canonical_json_hash(&value);
        if actual != file.hash {
            return Err(format!(
                "observation_blocked:projection_file_hash_mismatch:{}",
                file.path
            ));
        }
    }
    if projection_root.join("runtime-log.v1.json").exists() {
        return Err("observation_blocked:legacy_runtime_log_alias_file_present".to_string());
    }
    Ok(())
}

fn rollback_verification(projection_hash: &str) -> SqliteObservationRollbackVerification {
    SqliteObservationRollbackVerification {
        status: "rollback_recovery_verification_dry_run_only".to_string(),
        would_disable_db_read_cut: true,
        would_use_last_verified_json_projection: true,
        would_preserve_db_for_audit: true,
        would_require_supervisor_decision: true,
        production_restore_performed: false,
        selected_projection_hash: projection_hash.to_string(),
        instructions: vec![
            "would disable DB read-cut before any production recovery".to_string(),
            "would use the last verified JSON projection hash as recovery input".to_string(),
            "would preserve the SQLite DB for audit".to_string(),
            "would require supervisor decision before production restore".to_string(),
            "production restore is not performed by this fixture rehearsal".to_string(),
        ],
    }
}

fn rollback_manifest_value(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    manifest_path: &Path,
    source_root_hash: &str,
    projected_files: &[SqliteObservationProjectedFile],
    db_row_counts: &BTreeMap<String, i64>,
    redaction_policy: &[String],
    rollback_verification: &SqliteObservationRollbackVerification,
) -> Value {
    let payload = json!({
        "schema_version": "workbench_sqlite_observation_period.v1",
        "status": "completed",
        "observation_status": "stable_verified",
        "source_root_ref": fixture_root.display().to_string(),
        "source_root_hash": source_root_hash,
        "db_path_ref": db_path.display().to_string(),
        "projection_root_ref": projection_root.display().to_string(),
        "manifest_path_ref": manifest_path.display().to_string(),
        "projected_files": projected_files,
        "db_row_counts": db_row_counts,
        "redaction_policy": redaction_policy,
        "runtime_log_alias_policy": "canonical_runtime_logs_only",
        "rollback_recovery_verification": rollback_verification
    });
    let manifest_hash = canonical_json_hash(&payload);
    let mut manifest = payload;
    manifest["rollback_manifest_hash"] = Value::String(manifest_hash);
    manifest
}

fn commit_rollback_manifest(path: &Path, value: &Value) -> Result<(), String> {
    let temp_path = temp_manifest_path(path);
    remove_file_if_exists(&temp_path)?;
    write_json_file(&temp_path, value)?;
    fs::rename(&temp_path, path).map_err(|error| {
        format!(
            "commit sqlite observation rollback manifest failed {} -> {}: {error}",
            temp_path.display(),
            path.display()
        )
    })
}

fn verify_rollback_manifest(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("observation_blocked:rollback_manifest_missing:{error}"))?;
    let manifest: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("observation_blocked:rollback_manifest_corrupt:{error}"))?;
    if manifest.get("status").and_then(Value::as_str) != Some("completed") {
        remove_file_if_exists(path)?;
        return Err("observation_blocked:rollback_manifest_incomplete".to_string());
    }
    if manifest
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err("observation_blocked:rollback_manifest_hash_missing".to_string());
    }
    let restore_performed = manifest
        .get("rollback_recovery_verification")
        .and_then(|value| value.get("production_restore_performed"))
        .and_then(Value::as_bool);
    if restore_performed != Some(false) {
        return Err("observation_blocked:rollback_manifest_not_dry_run".to_string());
    }
    Ok(manifest)
}

fn verify_runtime_log_alias_policy(files: &[SqliteProjectedFile]) -> Result<(), String> {
    if files.iter().any(|file| file.path == "runtime-log.v1.json") {
        return Err("observation_blocked:legacy_runtime_log_alias_exported".to_string());
    }
    Ok(())
}

fn verify_db_integrity(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Err("observation_degraded:db_unavailable:missing_db_path".to_string());
    }
    let connection =
        Connection::open(db_path).map_err(|error| format!("observation_degraded:{error}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("observation_degraded:db_integrity_check_failed:{error}"))?;
    if integrity != "ok" {
        return Err(format!(
            "observation_degraded:db_integrity_check_failed:{integrity}"
        ));
    }
    let schema_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("observation_degraded:db_schema_mismatch:{error}"))?;
    if schema_count != 1 {
        return Err(
            "observation_degraded:db_schema_mismatch:missing_schema_migrations".to_string(),
        );
    }
    Ok(())
}

fn db_row_counts(db_path: &Path) -> Result<BTreeMap<String, i64>, String> {
    let mut counts = BTreeMap::new();
    for table in [
        "import_batches",
        "import_sources",
        "source_records",
        "projects",
        "workflows",
        "formal_memory_records",
        "memory_candidates",
        "observations",
        "product_commands",
        "session_continuations",
        "runtime_log_entries",
    ] {
        counts.insert(table.to_string(), table_count(db_path, table)?);
    }
    Ok(counts)
}

fn write_observation_report(
    path: &Path,
    report: &SqliteObservationRehearsalReport,
) -> Result<(), String> {
    if report.observation_status != "stable_verified" || !report.stable_verified || report.degraded
    {
        return Err(format!(
            "observation_report_status_not_committable:{}",
            report.observation_status
        ));
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize observation report value failed: {error}"))?,
    )
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("json path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("create json parent failed {}: {error}", parent.display()))?;
    let bytes =
        serde_json::to_vec(value).map_err(|error| format!("serialize json failed: {error}"))?;
    fs::write(path, bytes).map_err(|error| format!("write json failed {}: {error}", path.display()))
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove file failed {}: {error}", path.display())),
    }
}

fn remove_legacy_runtime_log_alias(projection_root: &Path) -> Result<(), String> {
    remove_file_if_exists(&projection_root.join("runtime-log.v1.json"))
}

fn temp_manifest_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
}

fn validate_fixture_root(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!("r3_a5_fixture_dir_required: {}", path.display()));
    }
    let fixture_root = manifest_r3_a5_fixture_root();
    if path.starts_with(&fixture_root) || path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_or_r3_a5_fixture_path_required: refusing fixture source outside temp or R3-A5 fixtures: {}",
            path.display()
        ))
    }
}

fn validate_temp_db_path(path: &Path) -> Result<(), String> {
    if path.is_absolute() && path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_db_path_required: refusing observation rehearsal db outside temp: {}",
            path.display()
        ))
    }
}

fn validate_projection_paths(
    projection_root: &Path,
    observation_report_path: &Path,
    rollback_manifest_path: &Path,
) -> Result<(), String> {
    if !projection_root.is_absolute()
        || !observation_report_path.is_absolute()
        || !rollback_manifest_path.is_absolute()
    {
        return Err(
            "temp_or_r3_a5_fixture_path_required: projection/report paths must be absolute".into(),
        );
    }
    let fixture_root = manifest_r3_a5_fixture_root();
    let projection_allowed = projection_root.starts_with(std::env::temp_dir())
        || projection_root.starts_with(fixture_root);
    if !projection_allowed
        || !observation_report_path.starts_with(projection_root)
        || !rollback_manifest_path.starts_with(projection_root)
    {
        return Err(format!(
            "temp_or_r3_a5_fixture_path_required: refusing projection/manifest/report outside temp or R3-A5 fixtures: {} / {} / {}",
            projection_root.display(),
            rollback_manifest_path.display(),
            observation_report_path.display()
        ));
    }
    Ok(())
}

fn manifest_r3_a5_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a5")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_observation_stable_verifies_two_samples_and_writes_report() {
        let fixture = fixture_dir("observation-export-valid-core-chain");
        let paths = prepare_paths("stable");

        let report = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            None,
        )
        .expect("stable observation");

        assert_eq!(report.observation_status, "stable_verified");
        assert!(report.stable_verified);
        assert!(!report.degraded);
        assert_eq!(report.sample_one.export_hash, report.sample_two.export_hash);
        assert_eq!(
            report.sample_one.projection_hash,
            report.sample_two.projection_hash
        );
        assert!(paths.observation_report_path.exists());
        assert!(paths.rollback_manifest_path.exists());
        assert!(paths.projection_root.join("runtime-logs.v1.json").exists());
        assert!(!paths.projection_root.join("runtime-log.v1.json").exists());
    }

    #[test]
    fn sqlite_observation_idempotent_rerun_keeps_stable_report_text() {
        let fixture = fixture_dir("observation-export-idempotent-rerun");
        let paths = prepare_paths("idempotent");

        rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            None,
        )
        .expect("first observation");
        let first_report =
            fs::read_to_string(&paths.observation_report_path).expect("first report");
        rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            None,
        )
        .expect("second observation");
        let second_report =
            fs::read_to_string(&paths.observation_report_path).expect("second report");

        assert_eq!(first_report, second_report);
    }

    #[test]
    fn sqlite_observation_export_hash_mismatch_blocks_without_stable_report() {
        let fixture = fixture_dir("observation-export-hash-mismatch-blocked");
        let paths = prepare_paths("export-hash-mismatch");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::ExportHashMismatch),
        )
        .expect_err("export hash mismatch must block");

        assert!(err.contains("observation_blocked:export_hash_mismatch"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_projection_missing_blocks_without_stable_report() {
        let fixture = fixture_dir("observation-projection-missing-blocked");
        let paths = prepare_paths("projection-missing");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::ProjectionFileMissing),
        )
        .expect_err("missing projection must block");

        assert!(err.contains("observation_blocked:projection_file_missing"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_projection_corrupt_blocks_without_stable_report() {
        let fixture = fixture_dir("observation-projection-missing-blocked");
        let paths = prepare_paths("projection-corrupt");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::ProjectionFileCorrupt),
        )
        .expect_err("corrupt projection must block");

        assert!(err.contains("observation_blocked:projection_file_corrupt"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_missing_manifest_blocks_without_stable_report() {
        let fixture = fixture_dir("observation-manifest-missing-blocked");
        let paths = prepare_paths("missing-manifest");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::RollbackManifestMissing),
        )
        .expect_err("missing manifest must block");

        assert!(err.contains("observation_blocked:rollback_manifest_missing"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_incomplete_manifest_blocks_without_stable_report() {
        let fixture = fixture_dir("observation-manifest-incomplete-blocked");
        let paths = prepare_paths("incomplete-manifest");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::RollbackManifestIncomplete),
        )
        .expect_err("incomplete manifest must block");

        assert!(err.contains("observation_blocked:rollback_manifest_incomplete"));
        assert!(!paths.observation_report_path.exists());
        assert!(!paths.rollback_manifest_path.exists());
    }

    #[test]
    fn sqlite_observation_db_integrity_failure_is_degraded_and_has_no_stable_report() {
        let fixture = fixture_dir("observation-db-integrity-failure-degraded");
        let paths = prepare_paths("db-integrity");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::DbIntegrityOrSchemaMismatch),
        )
        .expect_err("db integrity failure must degrade");

        assert!(err.contains("observation_degraded"));
        assert!(!err.contains("stable_verified"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_drift_between_samples_blocks_without_stable_report() {
        let fixture = fixture_dir("observation-export-valid-core-chain");
        let paths = prepare_paths("drift");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::ObservationDriftBetweenSamples),
        )
        .expect_err("observation drift must block");

        assert!(err.contains("observation_blocked:projection_hash_drift"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_failure_before_sample_creates_no_outputs() {
        let fixture = fixture_dir("observation-export-valid-core-chain");
        let paths = prepare_paths("before-sample");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::BeforeObservationSample),
        )
        .expect_err("before sample failure");

        assert!(err.contains("injected_failure_before_observation_sample"));
        assert!(!paths.db_path.exists());
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_failure_after_first_export_before_second_sample_creates_no_report() {
        let fixture = fixture_dir("observation-export-valid-core-chain");
        let paths = prepare_paths("after-first-export");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::AfterFirstExportBeforeSecondSample),
        )
        .expect_err("after first export failure");

        assert!(err.contains("after_first_export_before_second_sample"));
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_failure_after_rollback_selected_before_report_commit_creates_no_report() {
        let fixture = fixture_dir("rollback-export-recovery-verification-dry-run");
        let paths = prepare_paths("rollback-before-report");

        let err = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(SqliteObservationFailurePoint::AfterRollbackSelectedBeforeReportCommit),
        )
        .expect_err("after rollback selected failure");

        assert!(err.contains("after_rollback_selected_before_report_commit"));
        assert!(paths.rollback_manifest_path.exists());
        assert!(!paths.observation_report_path.exists());
    }

    #[test]
    fn sqlite_observation_rollback_verification_is_dry_run_only() {
        let fixture = fixture_dir("rollback-export-recovery-verification-dry-run");
        let paths = prepare_paths("rollback-dry-run");

        let report = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            None,
        )
        .expect("rollback dry-run");

        assert_eq!(
            report.rollback_recovery_verification.status,
            "rollback_recovery_verification_dry_run_only"
        );
        assert!(
            report
                .rollback_recovery_verification
                .would_disable_db_read_cut
        );
        assert!(
            report
                .rollback_recovery_verification
                .would_use_last_verified_json_projection
        );
        assert!(
            report
                .rollback_recovery_verification
                .would_preserve_db_for_audit
        );
        assert!(
            report
                .rollback_recovery_verification
                .would_require_supervisor_decision
        );
        assert!(
            !report
                .rollback_recovery_verification
                .production_restore_performed
        );
        let manifest_text =
            fs::read_to_string(&paths.rollback_manifest_path).expect("read manifest");
        assert!(manifest_text.contains("\"production_restore_performed\":false"));
        assert!(!manifest_text.contains("\"production_restore_performed\":true"));
    }

    #[test]
    fn sqlite_observation_report_projection_and_manifest_omit_forbidden_sensitive_fields() {
        let fixture = fixture_dir("observation-sensitive-redaction");
        let paths = prepare_paths("sensitive");

        rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            None,
        )
        .expect("sensitive observation");

        let mut text =
            fs::read_to_string(&paths.observation_report_path).expect("read observation report");
        text.push_str(&fs::read_to_string(&paths.rollback_manifest_path).expect("read manifest"));
        for entry in fs::read_dir(&paths.projection_root).expect("read projection") {
            let entry = entry.expect("projection entry");
            if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
                text.push_str(&fs::read_to_string(entry.path()).expect("read projection file"));
            }
        }
        assert!(!text.contains("provider credential value"));
        assert!(!text.contains("full transcript body"));
        assert!(!text.contains("rollout body payload"));
        assert!(!text.contains("\"prompt_body\""));
        assert!(text.contains("prompt_body:omitted"));
    }

    #[test]
    fn sqlite_observation_export_records_per_file_verification_fields() {
        let fixture = fixture_dir("observation-export-valid-core-chain");
        let paths = prepare_paths("per-file");

        let report = rehearse_fixture_observation_period(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            None,
        )
        .expect("per-file verification");

        for file in &report.export_verification.projected_files {
            assert!(!file.path.is_empty());
            assert!(file.hash.len() >= 64);
            assert!(file.record_count > 0 || file.path == "workflow-state.v0.json");
            assert_eq!(file.redaction_status, "forbidden_sensitive_fields_omitted");
        }
        assert!(report
            .export_verification
            .projected_files
            .iter()
            .any(|file| file.path == "runtime-logs.v1.json"));
        assert!(!report
            .export_verification
            .projected_files
            .iter()
            .any(|file| file.path == "runtime-log.v1.json"));
    }

    struct ObservationPaths {
        db_path: PathBuf,
        projection_root: PathBuf,
        observation_report_path: PathBuf,
        rollback_manifest_path: PathBuf,
    }

    fn prepare_paths(name: &str) -> ObservationPaths {
        let projection_root = temp_projection_root(name);
        ObservationPaths {
            db_path: temp_db(name),
            observation_report_path: projection_root.join("observation-report.json"),
            rollback_manifest_path: projection_root.join("rollback-manifest.json"),
            projection_root,
        }
    }

    fn fixture_dir(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("r3-a5")
            .join(name)
    }

    fn temp_db(name: &str) -> PathBuf {
        let nanos = unique_nanos();
        std::env::temp_dir().join(format!("r3-a5-{name}-{nanos}.sqlite"))
    }

    fn temp_projection_root(name: &str) -> PathBuf {
        let nanos = unique_nanos();
        std::env::temp_dir().join(format!("r3-a5-{name}-{nanos}"))
    }

    fn unique_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    }
}
