use crate::utils::fs_ops::remove_file_if_exists;
use crate::workbench_sqlite_apply::{apply_fixture_dir_to_temp_db, table_count};
use crate::workbench_sqlite_exporter::{export_temp_db_to_json_dry_run, SqliteProjectedFile};
use crate::workbench_sqlite_importer::canonical_json_hash;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteDualWriteFailurePoint {
    BeforeDbApply,
    AfterDbApplyBeforeProjectionWrite,
    AfterFirstProjectionFileBeforeManifest,
    BeforeManifestCommit,
    AfterManifestCommit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteDualWriteRehearsalReport {
    pub(crate) status: String,
    pub(crate) source_root_hash: String,
    pub(crate) db_path_ref: String,
    pub(crate) projection_root_ref: String,
    pub(crate) manifest_path_ref: String,
    pub(crate) projected_files: Vec<SqliteDualWriteProjectedFile>,
    pub(crate) db_row_counts: BTreeMap<String, i64>,
    pub(crate) rollback_manifest_hash: Option<String>,
    pub(crate) failure_point: Option<String>,
    pub(crate) recovery_dry_run_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteDualWriteProjectedFile {
    pub(crate) path: String,
    pub(crate) projected_hash: String,
    pub(crate) record_count: usize,
}

pub(crate) fn rehearse_fixture_dual_write(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    manifest_path: &Path,
    failure_point: Option<SqliteDualWriteFailurePoint>,
) -> Result<SqliteDualWriteRehearsalReport, String> {
    validate_fixture_root(fixture_root)?;
    validate_temp_db_path(db_path)?;
    validate_projection_paths(projection_root, manifest_path)?;

    if failure_point == Some(SqliteDualWriteFailurePoint::BeforeDbApply) {
        return Err("injected_failure_before_db_apply".to_string());
    }

    let apply_report = apply_fixture_dir_to_temp_db(fixture_root, db_path, None)?;
    if failure_point == Some(SqliteDualWriteFailurePoint::AfterDbApplyBeforeProjectionWrite) {
        return Err(format!(
            "projection_failed_after_db_commit:db_rows_committed:{}",
            apply_report.source_root_hash
        ));
    }

    let export_manifest =
        export_temp_db_to_json_dry_run(db_path, &projection_root.display().to_string())?;
    let projected_files = export_manifest
        .projected_files
        .iter()
        .map(|file| SqliteDualWriteProjectedFile {
            path: file.path.clone(),
            projected_hash: file.projected_hash.clone(),
            record_count: file.record_count,
        })
        .collect::<Vec<_>>();
    let db_row_counts = db_row_counts(db_path)?;

    fs::create_dir_all(projection_root).map_err(|error| {
        format!(
            "create sqlite dual-write projection root failed {}: {error}",
            projection_root.display()
        )
    })?;
    clear_incomplete_markers(projection_root, manifest_path)?;
    remove_file_if_exists(manifest_path)?;

    for (index, file) in export_manifest.projected_files.iter().enumerate() {
        write_projected_file(projection_root, file)?;
        if index == 0
            && failure_point
                == Some(SqliteDualWriteFailurePoint::AfterFirstProjectionFileBeforeManifest)
        {
            cleanup_projected_files(projection_root, &export_manifest.projected_files)?;
            write_json_file(
                &projection_root.join("projection-cleanup-incomplete.json"),
                &json!({
                    "schema_version": "workbench_sqlite_dual_write_rehearsal.v1",
                    "status": "projection_failed_before_manifest",
                    "cleanup": "partial_projection_files_removed",
                    "source_root_hash": apply_report.source_root_hash,
                    "failure_point": "AfterFirstProjectionFileBeforeManifest"
                }),
            )?;
            return Err("projection_failed_before_manifest:partial_projection_cleaned".to_string());
        }
    }

    let rollback_manifest = rollback_manifest_value(
        fixture_root,
        db_path,
        projection_root,
        manifest_path,
        &apply_report.source_root_hash,
        &projected_files,
        &db_row_counts,
        &export_manifest.redaction_manifest,
    );
    let rollback_manifest_hash = rollback_manifest
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    if failure_point == Some(SqliteDualWriteFailurePoint::BeforeManifestCommit) {
        let mut incomplete = rollback_manifest.clone();
        incomplete["status"] = Value::String("manifest_commit_failed_before_complete".to_string());
        incomplete["failure_point"] = Value::String("BeforeManifestCommit".to_string());
        write_json_file(
            &projection_root.join("rollback-manifest.incomplete.json"),
            &incomplete,
        )?;
        return Err("manifest_commit_failed:incomplete_manifest_written".to_string());
    }

    let temp_manifest = temp_manifest_path(manifest_path);
    write_json_file(&temp_manifest, &rollback_manifest)?;
    fs::rename(&temp_manifest, manifest_path).map_err(|error| {
        format!(
            "commit sqlite dual-write rollback manifest failed {} -> {}: {error}",
            temp_manifest.display(),
            manifest_path.display()
        )
    })?;

    let report = SqliteDualWriteRehearsalReport {
        status: "completed".to_string(),
        source_root_hash: apply_report.source_root_hash,
        db_path_ref: db_path.display().to_string(),
        projection_root_ref: projection_root.display().to_string(),
        manifest_path_ref: manifest_path.display().to_string(),
        projected_files,
        db_row_counts,
        rollback_manifest_hash: Some(rollback_manifest_hash),
        failure_point: failure_point.map(|point| format!("{point:?}")),
        recovery_dry_run_status: "recovery_dry_run_only".to_string(),
    };

    if failure_point == Some(SqliteDualWriteFailurePoint::AfterManifestCommit) {
        return Err(
            "injected_failure_after_manifest_commit:completed_manifest_available".to_string(),
        );
    }

    Ok(report)
}

fn validate_fixture_root(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!("r3_a3_fixture_dir_required: {}", path.display()));
    }
    let fixture_root = manifest_r3_a3_fixture_root();
    if path.starts_with(&fixture_root) || path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_or_r3_a3_fixture_path_required: refusing fixture source outside temp or R3-A3 fixtures: {}",
            path.display()
        ))
    }
}

fn validate_temp_db_path(path: &Path) -> Result<(), String> {
    if path.is_absolute() && path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_db_path_required: refusing dual-write rehearsal db outside temp: {}",
            path.display()
        ))
    }
}

fn validate_projection_paths(projection_root: &Path, manifest_path: &Path) -> Result<(), String> {
    if !projection_root.is_absolute() || !manifest_path.is_absolute() {
        return Err(
            "temp_or_r3_a3_fixture_path_required: projection paths must be absolute".into(),
        );
    }
    let fixture_root = manifest_r3_a3_fixture_root();
    let projection_allowed = projection_root.starts_with(std::env::temp_dir())
        || projection_root.starts_with(fixture_root);
    if !projection_allowed || !manifest_path.starts_with(projection_root) {
        return Err(format!(
            "temp_or_r3_a3_fixture_path_required: refusing projection/manifest outside temp or R3-A3 fixtures: {} / {}",
            projection_root.display(),
            manifest_path.display()
        ));
    }
    Ok(())
}

fn write_projected_file(root: &Path, file: &SqliteProjectedFile) -> Result<(), String> {
    if file.path.contains('/') || file.path.contains('\\') {
        return Err(format!("projection_file_name_must_be_flat: {}", file.path));
    }
    let path = root.join(&file.path);
    write_json_file(&path, &file.projection)
}

fn rollback_manifest_value(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    manifest_path: &Path,
    source_root_hash: &str,
    projected_files: &[SqliteDualWriteProjectedFile],
    db_row_counts: &BTreeMap<String, i64>,
    redaction_manifest: &[String],
) -> Value {
    let payload = json!({
        "schema_version": "workbench_sqlite_dual_write_rehearsal.v1",
        "status": "completed",
        "source_root_ref": fixture_root.display().to_string(),
        "source_root_hash": source_root_hash,
        "db_path_ref": db_path.display().to_string(),
        "projection_root_ref": projection_root.display().to_string(),
        "manifest_path_ref": manifest_path.display().to_string(),
        "projected_files": projected_files,
        "db_row_counts": db_row_counts,
        "redaction_policy": redaction_manifest,
        "recovery_instructions": [
            "recovery_dry_run_only",
            "verify projected file hashes before any restore decision",
            "do not restore or mutate production JSON from this fixture rehearsal"
        ],
        "recovery_dry_run": {
            "status": "recovery_dry_run_only",
            "would_remove_projection_root": projection_root.display().to_string(),
            "would_preserve_db_for_audit": db_path.display().to_string(),
            "production_restore_performed": false
        }
    });
    let payload_hash = canonical_json_hash(&payload);
    let mut manifest = payload;
    manifest["rollback_manifest_hash"] = Value::String(payload_hash);
    manifest
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
        "product_commands",
        "session_continuations",
        "runtime_log_entries",
    ] {
        counts.insert(table.to_string(), table_count(db_path, table)?);
    }
    Ok(counts)
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

fn cleanup_projected_files(root: &Path, files: &[SqliteProjectedFile]) -> Result<(), String> {
    for file in files {
        remove_file_if_exists(&root.join(&file.path))?;
    }
    Ok(())
}

fn clear_incomplete_markers(projection_root: &Path, manifest_path: &Path) -> Result<(), String> {
    remove_file_if_exists(&projection_root.join("projection-cleanup-incomplete.json"))?;
    remove_file_if_exists(&projection_root.join("rollback-manifest.incomplete.json"))?;
    remove_file_if_exists(&temp_manifest_path(manifest_path))
}

fn temp_manifest_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
}

fn manifest_r3_a3_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a3")
}

#[cfg(test)]
mod tests {
    use crate::utils::fs_ops::fixture_dir;

    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_dual_write_rehearsal_writes_projection_and_completed_manifest() {
        let fixture = fixture_dir("r3-a3", "dual-write-valid-core-chain");
        let db_path = temp_db("success");
        let projection_root = temp_projection_root("success");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let report =
            rehearse_fixture_dual_write(&fixture, &db_path, &projection_root, &manifest_path, None)
                .expect("dual-write rehearsal");

        assert_eq!(report.status, "completed");
        assert!(projection_root.join("workflow-state.v0.json").exists());
        assert!(projection_root.join("runtime-logs.v1.json").exists());
        assert!(manifest_path.exists());
        let manifest_text = fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(manifest_text.contains("\"status\":\"completed\""));
        assert!(manifest_text.contains("\"source_root_hash\""));
        assert!(manifest_text.contains("\"recovery_instructions\""));
    }

    #[test]
    fn sqlite_dual_write_rehearsal_is_idempotent_for_same_fixture_and_projection_root() {
        let fixture = fixture_dir("r3-a3", "dual-write-idempotent-rerun");
        let db_path = temp_db("idempotent");
        let projection_root = temp_projection_root("idempotent");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let first =
            rehearse_fixture_dual_write(&fixture, &db_path, &projection_root, &manifest_path, None)
                .expect("first rehearsal");
        let first_manifest = fs::read_to_string(&manifest_path).expect("first manifest");
        let second =
            rehearse_fixture_dual_write(&fixture, &db_path, &projection_root, &manifest_path, None)
                .expect("second rehearsal");
        let second_manifest = fs::read_to_string(&manifest_path).expect("second manifest");

        assert_eq!(first.status, "completed");
        assert_eq!(second.status, "completed");
        assert_eq!(first_manifest, second_manifest);
    }

    #[test]
    fn sqlite_dual_write_after_db_before_projection_failure_keeps_db_without_projection() {
        let fixture = fixture_dir("r3-a3", "dual-write-after-db-before-projection-failure");
        let db_path = temp_db("after-db-before-projection");
        let projection_root = temp_projection_root("after-db-before-projection");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let err = rehearse_fixture_dual_write(
            &fixture,
            &db_path,
            &projection_root,
            &manifest_path,
            Some(SqliteDualWriteFailurePoint::AfterDbApplyBeforeProjectionWrite),
        )
        .expect_err("projection failure after db commit");

        assert!(err.contains("projection_failed_after_db_commit"));
        assert!(db_path.exists(), "DB commit should remain visible");
        assert!(
            !projection_root.join("workflow-state.v0.json").exists(),
            "projection must not be partially written before projection begins"
        );
        assert!(!manifest_path.exists(), "manifest must not be completed");
    }

    #[test]
    fn sqlite_dual_write_before_db_apply_failure_creates_no_outputs() {
        let fixture = fixture_dir("r3-a3", "dual-write-valid-core-chain");
        let db_path = temp_db("before-db-apply");
        let projection_root = temp_projection_root("before-db-apply");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let err = rehearse_fixture_dual_write(
            &fixture,
            &db_path,
            &projection_root,
            &manifest_path,
            Some(SqliteDualWriteFailurePoint::BeforeDbApply),
        )
        .expect_err("before db apply failure");

        assert!(err.contains("injected_failure_before_db_apply"));
        assert!(!db_path.exists());
        assert!(!projection_root.exists());
    }

    #[test]
    fn sqlite_dual_write_projection_failure_cleans_partial_files_before_manifest() {
        let fixture = fixture_dir(
            "r3-a3",
            "dual-write-after-first-projection-before-manifest-failure",
        );
        let db_path = temp_db("projection-partial");
        let projection_root = temp_projection_root("projection-partial");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let err = rehearse_fixture_dual_write(
            &fixture,
            &db_path,
            &projection_root,
            &manifest_path,
            Some(SqliteDualWriteFailurePoint::AfterFirstProjectionFileBeforeManifest),
        )
        .expect_err("projection partial failure");

        assert!(err.contains("projection_failed_before_manifest"));
        assert!(!projection_root.join("workflow-state.v0.json").exists());
        assert!(projection_root
            .join("projection-cleanup-incomplete.json")
            .exists());
        assert!(!manifest_path.exists());
    }

    #[test]
    fn sqlite_dual_write_before_manifest_commit_marks_incomplete_without_completed_manifest() {
        let fixture = fixture_dir("r3-a3", "dual-write-before-manifest-commit-failure");
        let db_path = temp_db("manifest-incomplete");
        let projection_root = temp_projection_root("manifest-incomplete");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let err = rehearse_fixture_dual_write(
            &fixture,
            &db_path,
            &projection_root,
            &manifest_path,
            Some(SqliteDualWriteFailurePoint::BeforeManifestCommit),
        )
        .expect_err("manifest commit failure");

        assert!(err.contains("manifest_commit_failed"));
        assert!(projection_root
            .join("rollback-manifest.incomplete.json")
            .exists());
        assert!(!manifest_path.exists());
    }

    #[test]
    fn sqlite_dual_write_after_manifest_commit_keeps_completed_manifest_and_reports_failure() {
        let fixture = fixture_dir("r3-a3", "dual-write-after-manifest-commit");
        let db_path = temp_db("after-manifest");
        let projection_root = temp_projection_root("after-manifest");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let err = rehearse_fixture_dual_write(
            &fixture,
            &db_path,
            &projection_root,
            &manifest_path,
            Some(SqliteDualWriteFailurePoint::AfterManifestCommit),
        )
        .expect_err("after manifest commit injected failure");

        assert!(err.contains("injected_failure_after_manifest_commit"));
        assert!(manifest_path.exists());
        let manifest_text = fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(manifest_text.contains("\"status\":\"completed\""));
    }

    #[test]
    fn sqlite_dual_write_projection_redacts_forbidden_sensitive_fields() {
        let fixture = fixture_dir("r3-a3", "dual-write-sensitive-redaction");
        let db_path = temp_db("sensitive-redaction");
        let projection_root = temp_projection_root("sensitive-redaction");
        let manifest_path = projection_root.join("rollback-manifest.json");

        rehearse_fixture_dual_write(&fixture, &db_path, &projection_root, &manifest_path, None)
            .expect("dual-write redaction rehearsal");

        let projection_text = fs::read_to_string(projection_root.join("workflow-state.v0.json"))
            .expect("read projection");
        let all_projection_text = fs::read_dir(&projection_root)
            .expect("read projection dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name() != "rollback-manifest.json")
            .filter_map(|entry| fs::read_to_string(entry.path()).ok())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(projection_text.contains("workflow_state_v0"));
        assert!(!all_projection_text.contains("prompt_body"));
        assert!(!all_projection_text.contains("full_transcript"));
        assert!(!all_projection_text.contains("rollout_body"));
        assert!(!all_projection_text.contains("provider credential value"));
        let manifest_text = fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(manifest_text.contains("prompt_body:omitted"));
        assert!(manifest_text.contains("full_transcript:omitted"));
    }

    #[test]
    fn sqlite_dual_write_recovery_dry_run_reads_completed_manifest_without_mutating_files() {
        let fixture = fixture_dir("r3-a3", "rollback-manifest-recovery-dry-run");
        let db_path = temp_db("recovery");
        let projection_root = temp_projection_root("recovery");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let report =
            rehearse_fixture_dual_write(&fixture, &db_path, &projection_root, &manifest_path, None)
                .expect("dual-write rehearsal");

        assert_eq!(report.status, "completed");
        let manifest_text = fs::read_to_string(&manifest_path).expect("read manifest");
        assert!(manifest_text.contains("recovery_dry_run_only"));
        let recovered_outputs = fs::read_dir(&projection_root)
            .expect("read projection dir")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("recovered-")
            })
            .count();
        assert_eq!(recovered_outputs, 0);
    }

    #[test]
    fn sqlite_dual_write_rejects_non_temp_projection_root() {
        let fixture = fixture_dir("r3-a3", "dual-write-valid-core-chain");
        let db_path = temp_db("bad-projection-root");
        let projection_root = Path::new("/var/r3-a3-projection");
        let manifest_path = projection_root.join("rollback-manifest.json");

        let err =
            rehearse_fixture_dual_write(&fixture, &db_path, projection_root, &manifest_path, None)
                .expect_err("non-temp projection root should reject");

        assert!(err.contains("temp_or_r3_a3_fixture_path_required"));
    }

    fn temp_db(name: &str) -> std::path::PathBuf {
        let nanos = unique_nanos();
        std::env::temp_dir().join(format!("r3-a3-{name}-{nanos}.sqlite"))
    }

    fn temp_projection_root(name: &str) -> std::path::PathBuf {
        let nanos = unique_nanos();
        std::env::temp_dir().join(format!("r3-a3-{name}-{nanos}"))
    }

    fn unique_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    }
}
