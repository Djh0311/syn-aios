use crate::workbench_sqlite_apply::{
    apply_fixture_dir_to_temp_db, table_count, SqliteApplyFailurePoint,
};
use crate::workbench_sqlite_exporter::{export_temp_db_to_json_dry_run, SqliteProjectedFile};
use crate::workbench_sqlite_importer::{
    canonical_json_hash, OPTIONAL_SIDECARS, PRIMARY_WORKFLOW_STATE,
};
use crate::workbench_sqlite_preflight::{
    scan_workbench_state_root_preflight_with_config, SqliteProductionPreflightConfig,
    SqliteProductionPreflightReport,
};
use crate::workbench_sqlite_schema::WORKBENCH_SQLITE_SCHEMA_VERSION;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const MODE: &str = "production_apply";
const LEVEL_A: &str = "level_a_fixture";
const PRODUCTION_APPLY_SCHEMA_VERSION: &str = "workbench_sqlite_production_apply.v1";
const BACKUP_MANIFEST_NAME: &str = "production-apply-backup-manifest.json";
const APPLY_MANIFEST_NAME: &str = "production-apply-manifest.json";
const EXPORT_MANIFEST_NAME: &str = "production-apply-export-manifest.json";
const DEFAULT_DENIED_PATH_MARKERS: &[&str] = &[
    "/users/yoyi/.codex",
    ".codex",
    ".env",
    "token",
    "secret",
    "credential",
    "keychain",
    "oauth",
    "provider_credential",
    "provider credential",
    "full_transcript",
    "full transcript",
    "rollout",
    "prompt_body",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteProductionApplyFailurePoint {
    BackupManifestWriteFailureBeforeDbCreate,
    DbInitializeFailure,
    ImportRejectedCorruptSnapshot,
    TransactionRollbackBeforeCommit,
    AfterDbCommitBeforeManifestCommit,
    ExportHashMismatch,
    RollbackManifestMissing,
    RollbackManifestIncomplete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SqliteProductionApplyConfig {
    pub(crate) allowed_sidecars: BTreeSet<String>,
    pub(crate) denied_path_markers: Vec<String>,
    pub(crate) expected_source_root_hash: Option<String>,
    pub(crate) expected_preflight_report_hash: Option<String>,
    pub(crate) expected_copied_snapshot_report_hash: Option<String>,
}

impl Default for SqliteProductionApplyConfig {
    fn default() -> Self {
        Self {
            allowed_sidecars: OPTIONAL_SIDECARS
                .iter()
                .map(|sidecar| (*sidecar).to_string())
                .collect(),
            denied_path_markers: DEFAULT_DENIED_PATH_MARKERS
                .iter()
                .map(|marker| (*marker).to_string())
                .collect(),
            expected_source_root_hash: None,
            expected_preflight_report_hash: None,
            expected_copied_snapshot_report_hash: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionApplyReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) level: String,
    pub(crate) status: String,
    pub(crate) source_root_ref: String,
    pub(crate) source_root_hash: String,
    pub(crate) source_root_path_hash: String,
    pub(crate) production_db_path_hash: String,
    pub(crate) backup_root_ref: String,
    pub(crate) backup_root_hash: String,
    pub(crate) backup_manifest_hash: String,
    pub(crate) rollback_manifest_hash: String,
    pub(crate) rollback_manifest_path_hash: String,
    pub(crate) report_path_hash: String,
    pub(crate) preflight_report_hash: String,
    pub(crate) copied_snapshot_report_hash: Option<String>,
    pub(crate) db_schema_version: String,
    pub(crate) import_batch_id: String,
    pub(crate) import_batch_hash: String,
    pub(crate) table_counts: BTreeMap<String, i64>,
    pub(crate) source_record_counts: BTreeMap<String, usize>,
    pub(crate) export_verification: SqliteProductionExportVerification,
    pub(crate) rollback_boundary: SqliteProductionRollbackBoundary,
    pub(crate) before_source_hashes: Vec<SqliteProductionSourceFileHash>,
    pub(crate) after_source_hashes: Vec<SqliteProductionSourceFileHash>,
    pub(crate) safety_flags: SqliteProductionSafetyFlags,
    pub(crate) failure_point: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionSourceFileHash {
    pub(crate) path_ref: String,
    pub(crate) path_hash: String,
    pub(crate) file_hash: String,
    pub(crate) size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionExportVerification {
    pub(crate) status: String,
    pub(crate) db_export_hash: String,
    pub(crate) projection_hash: String,
    pub(crate) export_manifest_hash: String,
    pub(crate) runtime_log_alias_policy: String,
    pub(crate) projected_files: Vec<SqliteProductionProjectedFile>,
    pub(crate) redaction_manifest: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionProjectedFile {
    pub(crate) path: String,
    pub(crate) projected_hash: String,
    pub(crate) record_count: usize,
    pub(crate) redaction_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionRollbackBoundary {
    pub(crate) status: String,
    pub(crate) would_disable_db_read_cut: bool,
    pub(crate) would_preserve_db_for_audit: bool,
    pub(crate) would_use_source_backup: bool,
    pub(crate) would_use_last_export_projection: bool,
    pub(crate) would_require_supervisor_decision: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) instructions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionSafetyFlags {
    pub(crate) production_db_created: bool,
    pub(crate) production_root_written: bool,
    pub(crate) production_apply_performed: bool,
    pub(crate) read_cut_enabled: bool,
    pub(crate) stop_write_json: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) codex_home_touched: bool,
    pub(crate) product_read_path_changed: bool,
    pub(crate) source_json_written: bool,
}

impl SqliteProductionSafetyFlags {
    fn completed() -> Self {
        Self {
            production_db_created: true,
            production_root_written: false,
            production_apply_performed: true,
            read_cut_enabled: false,
            stop_write_json: false,
            production_restore_performed: false,
            codex_home_touched: false,
            product_read_path_changed: false,
            source_json_written: false,
        }
    }

    fn failed_after_commit() -> Self {
        Self {
            production_db_created: true,
            production_root_written: false,
            production_apply_performed: true,
            read_cut_enabled: false,
            stop_write_json: false,
            production_restore_performed: false,
            codex_home_touched: false,
            product_read_path_changed: false,
            source_json_written: false,
        }
    }
}

pub(crate) fn rehearse_production_db_apply_level_a(
    source_state_root: &Path,
    production_db_path: &Path,
    backup_root: &Path,
    report_path: &Path,
    rollback_manifest_path: &Path,
    config: &SqliteProductionApplyConfig,
    failure_point: Option<SqliteProductionApplyFailurePoint>,
) -> Result<SqliteProductionApplyReport, String> {
    let denied_path_markers = effective_denied_path_markers(config);
    validate_level_a_source_root(source_state_root, &denied_path_markers)?;
    validate_level_a_output_paths(
        source_state_root,
        production_db_path,
        backup_root,
        report_path,
        rollback_manifest_path,
        &denied_path_markers,
    )?;
    remove_file_if_exists(report_path)?;
    remove_file_if_exists(rollback_manifest_path)?;

    let preflight_config = preflight_config(config, &denied_path_markers);
    let preflight = scan_workbench_state_root_preflight_with_config(
        source_state_root,
        None,
        &preflight_config,
    )?;
    ensure_preflight_ready(&preflight)?;
    let preflight_report_hash = report_hash(&preflight)?;
    if let Some(expected) = &config.expected_source_root_hash {
        if &preflight.source_root_hash != expected {
            return Err(format!(
                "production_apply_blocked:source_root_hash_mismatch:expected={expected}:actual={}",
                preflight.source_root_hash
            ));
        }
    }
    if let Some(expected) = &config.expected_preflight_report_hash {
        if &preflight_report_hash != expected {
            return Err(format!(
                "production_apply_blocked:preflight_report_hash_mismatch:expected={expected}:actual={preflight_report_hash}"
            ));
        }
    }

    let before_source_hashes = source_file_hashes(source_state_root, config)?;
    reset_dir(backup_root)?;
    if failure_point
        == Some(SqliteProductionApplyFailurePoint::BackupManifestWriteFailureBeforeDbCreate)
    {
        let marker = backup_root.join("backup-manifest-write-failure.injected");
        fs::write(&marker, b"backup manifest write failure before db create").map_err(|error| {
            format!(
                "create backup manifest failure marker failed {}: {error}",
                marker.display()
            )
        })?;
        return Err("injected_failure_backup_manifest_write_before_db_create".to_string());
    }
    let backup_manifest_hash = write_backup_manifest(
        source_state_root,
        backup_root,
        &before_source_hashes,
        &preflight,
        &preflight_report_hash,
        config.expected_copied_snapshot_report_hash.as_deref(),
    )?;

    if failure_point == Some(SqliteProductionApplyFailurePoint::DbInitializeFailure) {
        fs::create_dir_all(production_db_path).map_err(|error| {
            format!(
                "create db initialize failure fixture dir failed {}: {error}",
                production_db_path.display()
            )
        })?;
        let err = apply_fixture_dir_to_temp_db(source_state_root, production_db_path, None)
            .expect_err("db initialize failure must error");
        return Err(format!(
            "production_apply_failed:db_initialize_failure:{err}"
        ));
    } else {
        remove_file_if_exists(production_db_path)?;
    }

    if failure_point == Some(SqliteProductionApplyFailurePoint::ImportRejectedCorruptSnapshot) {
        let temp_corrupt_root = corrupt_source_copy(source_state_root)?;
        return apply_failure_without_report(
            &temp_corrupt_root,
            production_db_path,
            Some("corrupt_snapshot"),
        );
    }
    if failure_point == Some(SqliteProductionApplyFailurePoint::TransactionRollbackBeforeCommit) {
        let result = apply_fixture_dir_to_temp_db(
            source_state_root,
            production_db_path,
            Some(SqliteApplyFailurePoint::BeforeCommit),
        );
        let err = result.expect_err("transaction rollback failure must error");
        let row_count = table_count(production_db_path, "import_batches").unwrap_or(0);
        if row_count != 0 {
            return Err(format!(
                "production_apply_blocked:transaction_rollback_left_partial_rows:{row_count}"
            ));
        }
        return Err(format!("production_apply_failed:{err}"));
    }

    let apply_report = apply_fixture_dir_to_temp_db(source_state_root, production_db_path, None)?;
    let table_counts = db_row_counts(production_db_path)?;
    let source_record_counts = source_record_counts(source_state_root, config)?;
    let import_batch_hash = canonical_json_hash(&json!({
        "batch_id": apply_report.batch_id,
        "source_root_hash": apply_report.source_root_hash,
        "records_inserted": apply_report.records_inserted,
        "records_skipped": apply_report.records_skipped,
        "sources_inserted": apply_report.sources_inserted,
        "table_counts": table_counts
    }));

    if failure_point == Some(SqliteProductionApplyFailurePoint::AfterDbCommitBeforeManifestCommit) {
        let after_source_hashes = source_file_hashes(source_state_root, config)?;
        let rollback_boundary = rollback_boundary();
        let export_verification = export_verification(production_db_path, backup_root)?;
        let failed_report = SqliteProductionApplyReport {
            schema_version: PRODUCTION_APPLY_SCHEMA_VERSION.to_string(),
            mode: MODE.to_string(),
            level: LEVEL_A.to_string(),
            status: "failed_classified".to_string(),
            source_root_ref: source_state_root.display().to_string(),
            source_root_hash: preflight.source_root_hash,
            source_root_path_hash: path_hash(source_state_root),
            production_db_path_hash: path_hash(production_db_path),
            backup_root_ref: backup_root.display().to_string(),
            backup_root_hash: dir_manifest_hash(backup_root)?,
            backup_manifest_hash,
            rollback_manifest_hash: "not_committed_after_db_commit_failure".to_string(),
            rollback_manifest_path_hash: path_hash(rollback_manifest_path),
            report_path_hash: path_hash(report_path),
            preflight_report_hash,
            copied_snapshot_report_hash: config.expected_copied_snapshot_report_hash.clone(),
            db_schema_version: WORKBENCH_SQLITE_SCHEMA_VERSION.to_string(),
            import_batch_id: apply_report.batch_id,
            import_batch_hash,
            table_counts,
            source_record_counts,
            export_verification,
            rollback_boundary,
            before_source_hashes,
            after_source_hashes,
            safety_flags: SqliteProductionSafetyFlags::failed_after_commit(),
            failure_point: failure_point.map(|point| format!("{point:?}")),
        };
        write_report(report_path, &failed_report, true)?;
        return Err("injected_failure_after_db_commit_before_manifest_commit".to_string());
    }

    let mut export_verification = export_verification(production_db_path, backup_root)?;
    if failure_point == Some(SqliteProductionApplyFailurePoint::ExportHashMismatch) {
        export_verification.db_export_hash = "injected_export_hash_mismatch".to_string();
    }
    verify_export_hashes(&export_verification)?;
    write_export_manifest(backup_root, &export_verification)?;

    let rollback_boundary = rollback_boundary();
    write_rollback_manifest(
        rollback_manifest_path,
        &rollback_boundary,
        &backup_manifest_hash,
    )?;
    if failure_point == Some(SqliteProductionApplyFailurePoint::RollbackManifestMissing) {
        remove_file_if_exists(rollback_manifest_path)?;
    }
    if failure_point == Some(SqliteProductionApplyFailurePoint::RollbackManifestIncomplete) {
        write_json_file(
            rollback_manifest_path,
            &json!({
                "schema_version": PRODUCTION_APPLY_SCHEMA_VERSION,
                "status": "manifest_commit_failed_before_complete",
                "production_restore_performed": false
            }),
        )?;
    }
    let rollback_manifest_hash = verify_rollback_manifest(rollback_manifest_path)?;

    let after_source_hashes = source_file_hashes(source_state_root, config)?;
    if before_source_hashes != after_source_hashes {
        return Err("production_apply_blocked:source_hashes_changed".to_string());
    }

    let report = SqliteProductionApplyReport {
        schema_version: PRODUCTION_APPLY_SCHEMA_VERSION.to_string(),
        mode: MODE.to_string(),
        level: LEVEL_A.to_string(),
        status: "completed".to_string(),
        source_root_ref: source_state_root.display().to_string(),
        source_root_hash: preflight.source_root_hash,
        source_root_path_hash: path_hash(source_state_root),
        production_db_path_hash: path_hash(production_db_path),
        backup_root_ref: backup_root.display().to_string(),
        backup_root_hash: dir_manifest_hash(backup_root)?,
        backup_manifest_hash,
        rollback_manifest_hash,
        rollback_manifest_path_hash: path_hash(rollback_manifest_path),
        report_path_hash: path_hash(report_path),
        preflight_report_hash,
        copied_snapshot_report_hash: config.expected_copied_snapshot_report_hash.clone(),
        db_schema_version: WORKBENCH_SQLITE_SCHEMA_VERSION.to_string(),
        import_batch_id: apply_report.batch_id,
        import_batch_hash,
        table_counts,
        source_record_counts,
        export_verification,
        rollback_boundary,
        before_source_hashes,
        after_source_hashes,
        safety_flags: SqliteProductionSafetyFlags::completed(),
        failure_point: failure_point.map(|point| format!("{point:?}")),
    };
    write_apply_manifest(backup_root, &report)?;
    write_report(report_path, &report, false)?;
    Ok(report)
}

fn apply_failure_without_report<T>(
    source_root: &Path,
    production_db_path: &Path,
    label: Option<&str>,
) -> Result<T, String> {
    let err = apply_fixture_dir_to_temp_db(source_root, production_db_path, None)
        .expect_err("apply failure must error");
    Err(format!(
        "production_apply_failed:{}:{err}",
        label.unwrap_or("apply")
    ))
}

fn preflight_config(
    config: &SqliteProductionApplyConfig,
    denied_path_markers: &[String],
) -> SqliteProductionPreflightConfig {
    SqliteProductionPreflightConfig {
        primary_workflow_state: PRIMARY_WORKFLOW_STATE.to_string(),
        allowed_sidecars: config.allowed_sidecars.clone(),
        denied_path_markers: denied_path_markers.to_vec(),
    }
}

fn ensure_preflight_ready(report: &SqliteProductionPreflightReport) -> Result<(), String> {
    if report.status != "preflight_ready" || report.counts.blocked_reasons > 0 {
        return Err(format!(
            "production_apply_blocked:preflight_not_ready:status={}:blocked={}",
            report.status, report.counts.blocked_reasons
        ));
    }
    if report.production_db_created
        || report.production_root_written
        || report.read_cut_enabled
        || report.stop_write_json
        || report.codex_home_touched
    {
        return Err("production_apply_blocked:preflight_flags_not_false".to_string());
    }
    Ok(())
}

fn write_backup_manifest(
    source_state_root: &Path,
    backup_root: &Path,
    before_hashes: &[SqliteProductionSourceFileHash],
    preflight: &SqliteProductionPreflightReport,
    preflight_report_hash: &str,
    copied_snapshot_report_hash: Option<&str>,
) -> Result<String, String> {
    let manifest = json!({
        "schema_version": PRODUCTION_APPLY_SCHEMA_VERSION,
        "mode": MODE,
        "level": LEVEL_A,
        "status": "completed",
        "source_root_ref": source_state_root.display().to_string(),
        "source_root_hash": preflight.source_root_hash,
        "preflight_report_hash": preflight_report_hash,
        "copied_snapshot_report_hash": copied_snapshot_report_hash,
        "source_files": before_hashes,
        "source_json_written": false,
        "production_root_written": false
    });
    let manifest_hash = canonical_json_hash(&manifest);
    let mut value = manifest;
    value["backup_manifest_hash"] = Value::String(manifest_hash.clone());
    write_json_file(&backup_root.join(BACKUP_MANIFEST_NAME), &value)?;
    Ok(manifest_hash)
}

fn write_apply_manifest(
    backup_root: &Path,
    report: &SqliteProductionApplyReport,
) -> Result<(), String> {
    let report_value = serde_json::to_value(report)
        .map_err(|error| format!("serialize apply manifest report hash failed: {error}"))?;
    write_json_file(
        &backup_root.join(APPLY_MANIFEST_NAME),
        &json!({
            "schema_version": PRODUCTION_APPLY_SCHEMA_VERSION,
            "status": report.status,
            "mode": report.mode,
            "level": report.level,
            "source_root_hash": report.source_root_hash,
            "production_db_path_hash": report.production_db_path_hash,
            "import_batch_id": report.import_batch_id,
            "import_batch_hash": report.import_batch_hash,
            "safety_flags": report.safety_flags,
            "manifest_hash": canonical_json_hash(&report_value)
        }),
    )
}

fn export_verification(
    production_db_path: &Path,
    backup_root: &Path,
) -> Result<SqliteProductionExportVerification, String> {
    verify_db_integrity(production_db_path)?;
    let manifest =
        export_temp_db_to_json_dry_run(production_db_path, &backup_root.display().to_string())?;
    verify_runtime_log_alias_policy(&manifest.projected_files)?;
    let projected_files = production_projected_files(&manifest.projected_files);
    let projection_hash = projection_hash(&projected_files);
    Ok(SqliteProductionExportVerification {
        status: "verified".to_string(),
        db_export_hash: manifest.export_hash.clone(),
        projection_hash: projection_hash.clone(),
        export_manifest_hash: canonical_json_hash(&json!({
            "export_hash": manifest.export_hash,
            "projection_hash": projection_hash,
            "projected_files": projected_files,
            "redaction_manifest": manifest.redaction_manifest,
            "runtime_log_alias_policy": manifest.runtime_log_alias_policy
        })),
        runtime_log_alias_policy: manifest.runtime_log_alias_policy,
        projected_files,
        redaction_manifest: manifest.redaction_manifest,
    })
}

fn verify_export_hashes(verification: &SqliteProductionExportVerification) -> Result<(), String> {
    if verification.db_export_hash == "injected_export_hash_mismatch" {
        return Err("production_apply_blocked:export_hash_mismatch".to_string());
    }
    if verification
        .projected_files
        .iter()
        .any(|file| file.path == "runtime-log.v1.json")
    {
        return Err("production_apply_blocked:legacy_runtime_log_alias_exported".to_string());
    }
    Ok(())
}

fn write_export_manifest(
    backup_root: &Path,
    verification: &SqliteProductionExportVerification,
) -> Result<(), String> {
    let verification_value = serde_json::to_value(verification)
        .map_err(|error| format!("serialize export manifest hash failed: {error}"))?;
    write_json_file(
        &backup_root.join(EXPORT_MANIFEST_NAME),
        &json!({
            "schema_version": PRODUCTION_APPLY_SCHEMA_VERSION,
            "status": "completed",
            "mode": MODE,
            "export_verification": verification,
            "manifest_hash": canonical_json_hash(&verification_value)
        }),
    )
}

fn rollback_boundary() -> SqliteProductionRollbackBoundary {
    SqliteProductionRollbackBoundary {
        status: "rollback_boundary_dry_run_only".to_string(),
        would_disable_db_read_cut: true,
        would_preserve_db_for_audit: true,
        would_use_source_backup: true,
        would_use_last_export_projection: true,
        would_require_supervisor_decision: true,
        production_restore_performed: false,
        instructions: vec![
            "would disable DB read-cut before rollback".to_string(),
            "would preserve the DB file for audit".to_string(),
            "would use source backup manifest and last export projection".to_string(),
            "would require supervisor decision before production restore".to_string(),
            "production restore is not performed by this rehearsal".to_string(),
        ],
    }
}

fn write_rollback_manifest(
    rollback_manifest_path: &Path,
    boundary: &SqliteProductionRollbackBoundary,
    backup_manifest_hash: &str,
) -> Result<String, String> {
    let manifest = json!({
        "schema_version": PRODUCTION_APPLY_SCHEMA_VERSION,
        "status": "completed",
        "mode": MODE,
        "level": LEVEL_A,
        "backup_manifest_hash": backup_manifest_hash,
        "rollback_boundary": boundary,
        "production_restore_performed": false
    });
    let manifest_hash = canonical_json_hash(&manifest);
    let mut value = manifest;
    value["rollback_manifest_hash"] = Value::String(manifest_hash.clone());
    write_json_file(rollback_manifest_path, &value)?;
    Ok(manifest_hash)
}

fn verify_rollback_manifest(rollback_manifest_path: &Path) -> Result<String, String> {
    let bytes = fs::read(rollback_manifest_path)
        .map_err(|error| format!("production_apply_blocked:rollback_manifest_missing:{error}"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("production_apply_blocked:rollback_manifest_corrupt:{error}"))?;
    if value.get("status").and_then(Value::as_str) != Some("completed") {
        remove_file_if_exists(rollback_manifest_path)?;
        return Err("production_apply_blocked:rollback_manifest_incomplete".to_string());
    }
    let restore_performed = value
        .get("rollback_boundary")
        .and_then(|boundary| boundary.get("production_restore_performed"))
        .and_then(Value::as_bool);
    if restore_performed != Some(false) {
        return Err("production_apply_blocked:rollback_manifest_not_dry_run".to_string());
    }
    value
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| "production_apply_blocked:rollback_manifest_hash_missing".to_string())
}

fn source_file_hashes(
    source_state_root: &Path,
    config: &SqliteProductionApplyConfig,
) -> Result<Vec<SqliteProductionSourceFileHash>, String> {
    let mut names = vec![PRIMARY_WORKFLOW_STATE.to_string()];
    names.extend(config.allowed_sidecars.iter().cloned());
    names.sort();
    names.dedup();
    let mut hashes = Vec::new();
    for name in names {
        if name.contains('/') || name.contains('\\') {
            return Err(format!(
                "production_apply_blocked:sidecar_name_must_be_flat:{name}"
            ));
        }
        let path = source_state_root.join(&name);
        if !path.exists() {
            continue;
        }
        let bytes = fs::read(&path)
            .map_err(|error| format!("production_apply_source_read_failed:{name}:{error}"))?;
        hashes.push(SqliteProductionSourceFileHash {
            path_ref: name.clone(),
            path_hash: canonical_json_hash(&json!({ "path_ref": name })),
            file_hash: sha256_hex(&bytes),
            size_bytes: bytes.len() as u64,
        });
    }
    Ok(hashes)
}

fn source_record_counts(
    source_state_root: &Path,
    config: &SqliteProductionApplyConfig,
) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for file in source_file_hashes(source_state_root, config)? {
        let path = source_state_root.join(&file.path_ref);
        let bytes = fs::read(&path)
            .map_err(|error| format!("production_apply_source_count_read_failed:{error}"))?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("production_apply_source_count_parse_failed:{error}"))?;
        counts.insert(file.path_ref, record_count_estimate(&value));
    }
    Ok(counts)
}

fn record_count_estimate(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.len(),
        Value::Object(object) => object
            .values()
            .map(|value| match value {
                Value::Array(items) => items.len(),
                Value::Object(items) => items.len(),
                _ => 0,
            })
            .sum(),
        _ => 0,
    }
}

fn production_projected_files(files: &[SqliteProjectedFile]) -> Vec<SqliteProductionProjectedFile> {
    files
        .iter()
        .map(|file| SqliteProductionProjectedFile {
            path: file.path.clone(),
            projected_hash: file.projected_hash.clone(),
            record_count: file.record_count,
            redaction_status: "forbidden_sensitive_fields_omitted".to_string(),
        })
        .collect()
}

fn projection_hash(files: &[SqliteProductionProjectedFile]) -> String {
    canonical_json_hash(&json!({ "projected_files": files }))
}

fn verify_runtime_log_alias_policy(files: &[SqliteProjectedFile]) -> Result<(), String> {
    if files.iter().any(|file| file.path == "runtime-log.v1.json") {
        return Err("production_apply_blocked:legacy_runtime_log_alias_exported".to_string());
    }
    if !files.iter().any(|file| file.path == "runtime-logs.v1.json") {
        return Err(
            "production_apply_blocked:canonical_runtime_logs_projection_missing".to_string(),
        );
    }
    Ok(())
}

fn verify_db_integrity(production_db_path: &Path) -> Result<(), String> {
    if !production_db_path.exists() {
        return Err("production_apply_blocked:db_missing".to_string());
    }
    let connection = Connection::open(production_db_path)
        .map_err(|error| format!("production_apply_blocked:db_open_failed:{error}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("production_apply_blocked:db_integrity_failed:{error}"))?;
    if integrity != "ok" {
        return Err(format!(
            "production_apply_blocked:db_integrity_failed:{integrity}"
        ));
    }
    Ok(())
}

fn db_row_counts(db_path: &Path) -> Result<BTreeMap<String, i64>, String> {
    let mut counts = BTreeMap::new();
    for table in [
        "schema_migrations",
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

fn validate_level_a_source_root(path: &Path, denied_path_markers: &[String]) -> Result<(), String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!(
            "production_apply_source_root_required:{}",
            path.display()
        ));
    }
    if denied_path_hit(path, denied_path_markers) {
        return Err(format!(
            "production_apply_source_root_denied:{}",
            path.display()
        ));
    }
    let fixture_root = manifest_r3_a9_fixture_root();
    if path.starts_with(fixture_root) || path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "production_apply_level_a_source_required: refusing source outside temp or R3-A9 fixtures: {}",
            path.display()
        ))
    }
}

fn validate_level_a_output_paths(
    source_state_root: &Path,
    production_db_path: &Path,
    backup_root: &Path,
    report_path: &Path,
    rollback_manifest_path: &Path,
    denied_path_markers: &[String],
) -> Result<(), String> {
    for path in [
        production_db_path,
        backup_root,
        report_path,
        rollback_manifest_path,
    ] {
        if !path.is_absolute() {
            return Err(format!(
                "production_apply_absolute_path_required:{}",
                path.display()
            ));
        }
        if path.starts_with(source_state_root) {
            return Err(format!(
                "production_apply_blocked:path_inside_source_root_denied:{}",
                path.display()
            ));
        }
        if denied_path_hit(path, denied_path_markers) {
            return Err(format!(
                "production_apply_blocked:denied_path_marker:{}",
                path.display()
            ));
        }
    }
    if !production_db_path.starts_with(std::env::temp_dir())
        || !backup_root.starts_with(std::env::temp_dir())
        || !report_path.starts_with(std::env::temp_dir())
        || !rollback_manifest_path.starts_with(std::env::temp_dir())
    {
        return Err(
            "production_apply_temp_paths_required: db/backup/report/rollback must stay under temp"
                .to_string(),
        );
    }
    if report_path.starts_with(backup_root) || rollback_manifest_path.starts_with(source_state_root)
    {
        return Err("production_apply_report_or_rollback_path_invalid".to_string());
    }
    Ok(())
}

fn effective_denied_path_markers(config: &SqliteProductionApplyConfig) -> Vec<String> {
    let mut markers = DEFAULT_DENIED_PATH_MARKERS
        .iter()
        .map(|marker| marker.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    markers.extend(
        config
            .denied_path_markers
            .iter()
            .map(|marker| marker.to_ascii_lowercase()),
    );
    markers.into_iter().collect()
}

fn denied_path_hit(path: &Path, denied_path_markers: &[String]) -> bool {
    let haystack = path.to_string_lossy().to_ascii_lowercase();
    denied_path_markers
        .iter()
        .any(|marker| haystack.contains(&marker.to_ascii_lowercase()))
}

fn corrupt_source_copy(source_state_root: &Path) -> Result<PathBuf, String> {
    let root = std::env::temp_dir().join(format!(
        "workbench-r3-a9-corrupt-source-{}",
        std::process::id()
    ));
    reset_dir(&root)?;
    for entry in sorted_entries(source_state_root)? {
        let path = entry.path();
        if path.is_file() {
            fs::copy(&path, root.join(entry.file_name())).map_err(|error| {
                format!(
                    "copy corrupt source fixture failed {}: {error}",
                    path.display()
                )
            })?;
        }
    }
    fs::write(root.join(PRIMARY_WORKFLOW_STATE), b"{corrupt").map_err(|error| {
        format!(
            "write corrupt source fixture failed {}: {error}",
            root.join(PRIMARY_WORKFLOW_STATE).display()
        )
    })?;
    Ok(root)
}

fn write_report(
    path: &Path,
    report: &SqliteProductionApplyReport,
    allow_failed_classified: bool,
) -> Result<(), String> {
    if report.status != "completed"
        && !(allow_failed_classified && report.status == "failed_classified")
    {
        return Err(format!(
            "production_apply_report_status_not_committable:{}",
            report.status
        ));
    }
    if report.safety_flags.read_cut_enabled
        || report.safety_flags.stop_write_json
        || report.safety_flags.production_restore_performed
        || report.safety_flags.codex_home_touched
        || report.safety_flags.product_read_path_changed
        || report.safety_flags.source_json_written
        || report.safety_flags.production_root_written
    {
        return Err("production_apply_report_forbidden_flag_true".to_string());
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize production apply report failed: {error}"))?,
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

fn reset_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("remove temp dir failed {}: {error}", path.display()))?;
    }
    fs::create_dir_all(path)
        .map_err(|error| format!("create temp dir failed {}: {error}", path.display()))
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove file failed {}: {error}", path.display())),
    }
}

fn sorted_entries(root: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("read dir failed {}: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read dir entry failed {}: {error}", root.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

fn dir_manifest_hash(root: &Path) -> Result<String, String> {
    if !root.exists() {
        return Ok(canonical_json_hash(&json!({ "entries": [] })));
    }
    let mut entries = Vec::new();
    for entry in sorted_entries(root)? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let bytes = fs::read(&path)
            .map_err(|error| format!("production_apply_dir_hash_read_failed:{name}:{error}"))?;
        entries.push(json!({
            "path": name,
            "hash": sha256_hex(&bytes),
            "size_bytes": bytes.len()
        }));
    }
    Ok(canonical_json_hash(&json!({ "entries": entries })))
}

fn report_hash<T: Serialize>(value: &T) -> Result<String, String> {
    let value = serde_json::to_value(value)
        .map_err(|error| format!("serialize report hash failed: {error}"))?;
    Ok(canonical_json_hash(&value))
}

fn path_hash(path: &Path) -> String {
    canonical_json_hash(&json!({ "path": path.display().to_string() }))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn manifest_r3_a9_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a9")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_production_level_a_applies_temp_db_with_backup_and_report() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("success");

        let report = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            None,
        )
        .expect("production apply success");

        assert_eq!(report.mode, MODE);
        assert_eq!(report.level, LEVEL_A);
        assert_eq!(report.status, "completed");
        assert!(report.safety_flags.production_db_created);
        assert!(report.safety_flags.production_apply_performed);
        assert!(!report.safety_flags.read_cut_enabled);
        assert!(!report.safety_flags.stop_write_json);
        assert!(!report.safety_flags.source_json_written);
        assert_eq!(report.before_source_hashes, report.after_source_hashes);
        assert!(paths.backup_root.join(BACKUP_MANIFEST_NAME).exists());
        assert!(paths.backup_root.join(APPLY_MANIFEST_NAME).exists());
        assert!(paths.backup_root.join(EXPORT_MANIFEST_NAME).exists());
        assert!(paths.rollback_manifest_path.exists());
        assert!(paths.report_path.exists());
        assert!(report
            .export_verification
            .projected_files
            .iter()
            .any(|file| file.path == "runtime-logs.v1.json"));
    }

    #[test]
    fn sqlite_production_level_a_idempotent_rerun_is_deterministic() {
        let source = fixture_dir("production-idempotent-rerun");
        let paths = prepare_paths("idempotent");
        let config = SqliteProductionApplyConfig::default();

        rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect("first apply");
        let first = fs::read_to_string(&paths.report_path).expect("first report");
        rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect("second apply");
        let second = fs::read_to_string(&paths.report_path).expect("second report");

        assert_eq!(first, second);
    }

    #[test]
    fn sqlite_production_preflight_blocked_creates_no_db_or_report() {
        let source = fixture_dir("production-preflight-blocked-denied-path");
        let paths = prepare_paths("preflight-blocked");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            None,
        )
        .expect_err("preflight blocks");

        assert!(err.contains("preflight_not_ready"));
        assert!(!paths.db_path.exists());
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_production_backup_manifest_failure_happens_before_db_create() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("backup-failure");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            Some(SqliteProductionApplyFailurePoint::BackupManifestWriteFailureBeforeDbCreate),
        )
        .expect_err("backup manifest failure");

        assert!(err.contains("backup_manifest_write_before_db_create"));
        assert!(!paths.db_path.exists());
        assert!(!paths.report_path.exists());
        assert!(paths
            .backup_root
            .join("backup-manifest-write-failure.injected")
            .exists());
        assert!(!paths.backup_root.join(BACKUP_MANIFEST_NAME).exists());
    }

    #[test]
    fn sqlite_production_rejects_db_backup_report_and_rollback_inside_source_root() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("inside-source");
        for (db_path, backup_root, report_path, rollback_path) in [
            (
                source.join("db.sqlite"),
                paths.backup_root.clone(),
                paths.report_path.clone(),
                paths.rollback_manifest_path.clone(),
            ),
            (
                paths.db_path.clone(),
                source.join("backup"),
                paths.report_path.clone(),
                paths.rollback_manifest_path.clone(),
            ),
            (
                paths.db_path.clone(),
                paths.backup_root.clone(),
                source.join("report.json"),
                paths.rollback_manifest_path.clone(),
            ),
            (
                paths.db_path.clone(),
                paths.backup_root.clone(),
                paths.report_path.clone(),
                source.join("rollback.json"),
            ),
        ] {
            let err = rehearse_production_db_apply_level_a(
                &source,
                &db_path,
                &backup_root,
                &report_path,
                &rollback_path,
                &SqliteProductionApplyConfig::default(),
                None,
            )
            .expect_err("inside source path must reject");
            assert!(err.contains("path_inside_source_root_denied"));
        }
    }

    #[test]
    fn sqlite_production_db_initialize_failure_creates_no_completed_report() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("db-init-failure");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            Some(SqliteProductionApplyFailurePoint::DbInitializeFailure),
        )
        .expect_err("db initialize failure");

        assert!(err.contains("production_apply_failed:db_initialize_failure"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_production_import_rejected_corrupt_snapshot_creates_no_completed_report() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("import-corrupt");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            Some(SqliteProductionApplyFailurePoint::ImportRejectedCorruptSnapshot),
        )
        .expect_err("corrupt snapshot");

        assert!(err.contains("production_apply_failed:corrupt_snapshot"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_production_transaction_rollback_before_commit_leaves_no_partial_rows() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("rollback-before-commit");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            Some(SqliteProductionApplyFailurePoint::TransactionRollbackBeforeCommit),
        )
        .expect_err("transaction rollback");

        assert!(err.contains("injected_failure_before_commit"));
        assert!(!paths.report_path.exists());
        assert_eq!(
            table_count(&paths.db_path, "import_batches").unwrap_or(0),
            0
        );
    }

    #[test]
    fn sqlite_production_after_commit_failure_writes_failed_classified_report() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("after-commit-failure");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            Some(SqliteProductionApplyFailurePoint::AfterDbCommitBeforeManifestCommit),
        )
        .expect_err("after commit failure");

        assert!(err.contains("after_db_commit_before_manifest_commit"));
        assert!(paths.db_path.exists());
        assert!(paths.report_path.exists());
        let report: SqliteProductionApplyReport =
            serde_json::from_slice(&fs::read(&paths.report_path).expect("failed report"))
                .expect("parse failed report");
        assert_eq!(report.status, "failed_classified");
        assert!(report.safety_flags.production_apply_performed);
        assert!(!report.safety_flags.read_cut_enabled);
    }

    #[test]
    fn sqlite_production_export_hash_mismatch_blocks_completed_report() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("export-mismatch");

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            Some(SqliteProductionApplyFailurePoint::ExportHashMismatch),
        )
        .expect_err("export mismatch");

        assert!(err.contains("export_hash_mismatch"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_production_missing_or_incomplete_rollback_manifest_blocks_completion() {
        let source = fixture_dir("production-valid-core-chain");
        for (label, failure, expected) in [
            (
                "rollback-missing",
                SqliteProductionApplyFailurePoint::RollbackManifestMissing,
                "rollback_manifest_missing",
            ),
            (
                "rollback-incomplete",
                SqliteProductionApplyFailurePoint::RollbackManifestIncomplete,
                "rollback_manifest_incomplete",
            ),
        ] {
            let paths = prepare_paths(label);
            let err = rehearse_production_db_apply_level_a(
                &source,
                &paths.db_path,
                &paths.backup_root,
                &paths.report_path,
                &paths.rollback_manifest_path,
                &SqliteProductionApplyConfig::default(),
                Some(failure),
            )
            .expect_err("rollback manifest failure");
            assert!(err.contains(expected));
            assert!(!paths.report_path.exists());
        }
    }

    #[test]
    fn sqlite_production_expected_hash_mismatches_block() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = prepare_paths("expected-hash");
        let config = SqliteProductionApplyConfig {
            expected_source_root_hash: Some("wrong".to_string()),
            ..SqliteProductionApplyConfig::default()
        };

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect_err("expected hash mismatch");

        assert!(err.contains("source_root_hash_mismatch"));
    }

    struct Paths {
        db_path: PathBuf,
        backup_root: PathBuf,
        report_path: PathBuf,
        rollback_manifest_path: PathBuf,
    }

    fn fixture_dir(name: &str) -> PathBuf {
        manifest_r3_a9_fixture_root().join(name)
    }

    fn prepare_paths(label: &str) -> Paths {
        let unique = unique_label(label);
        let root = std::env::temp_dir().join(format!("workbench-r3-a9-{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root");
        Paths {
            db_path: root.join("production.sqlite"),
            backup_root: root.join("backup"),
            report_path: root.join("reports").join("production-apply-report.json"),
            rollback_manifest_path: root.join("rollback").join("rollback-manifest.json"),
        }
    }

    fn unique_label(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        format!("{label}-{nanos}")
    }
}
