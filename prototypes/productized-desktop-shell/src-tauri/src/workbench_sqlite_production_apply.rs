use crate::utils::fs_ops::remove_file_if_exists;
use crate::utils::hash::sha256_hex_bytes as sha256_hex;
use crate::workbench_sqlite_apply::{
    apply_confirmed_workbench_state_root_to_confirmed_db, apply_fixture_dir_to_temp_db,
    table_count, SqliteApplyFailurePoint, SqliteApplyImportReport,
};
use crate::workbench_sqlite_exporter::{
    export_confirmed_db_to_json_dry_run, export_temp_db_to_json_dry_run,
    SqliteExportDryRunManifest, SqliteProjectedFile,
};
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
const LEVEL_B_WORKBENCH_OWNED_STATE: &str = "level_b_workbench_owned_state";
const PRODUCTION_APPLY_SCHEMA_VERSION: &str = "workbench_sqlite_production_apply.v1";
const BACKUP_MANIFEST_NAME: &str = "production-apply-backup-manifest.json";
const APPLY_MANIFEST_NAME: &str = "production-apply-manifest.json";
const EXPORT_MANIFEST_NAME: &str = "production-apply-export-manifest.json";
const BACKUP_SOURCE_DIR_NAME: &str = "source-files";
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SqliteProductionApplyLevelBConfig {
    pub(crate) apply_config: SqliteProductionApplyConfig,
    pub(crate) confirmed_source_state_root: PathBuf,
    pub(crate) confirmed_production_db_path: PathBuf,
    pub(crate) confirmed_backup_root: PathBuf,
    pub(crate) confirmed_report_path: PathBuf,
    pub(crate) confirmed_rollback_manifest_path: PathBuf,
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
    rehearse_production_db_apply(
        LEVEL_A,
        source_state_root,
        production_db_path,
        backup_root,
        report_path,
        rollback_manifest_path,
        config,
        failure_point,
        |source, denied| validate_level_a_source_root(source, denied),
        |source, db, backup, report, rollback, denied| {
            validate_level_a_output_paths(source, db, backup, report, rollback, denied)
        },
        |source, db, failure| apply_fixture_dir_to_temp_db(source, db, failure),
        |db, backup| export_verification(db, backup),
    )
}

pub(crate) fn rehearse_production_db_apply_level_b_workbench_owned_state(
    source_state_root: &Path,
    production_db_path: &Path,
    backup_root: &Path,
    report_path: &Path,
    rollback_manifest_path: &Path,
    config: &SqliteProductionApplyLevelBConfig,
    failure_point: Option<SqliteProductionApplyFailurePoint>,
) -> Result<SqliteProductionApplyReport, String> {
    rehearse_production_db_apply(
        LEVEL_B_WORKBENCH_OWNED_STATE,
        source_state_root,
        production_db_path,
        backup_root,
        report_path,
        rollback_manifest_path,
        &config.apply_config,
        failure_point,
        |source, denied| {
            validate_level_b_source_root(source, &config.confirmed_source_state_root, denied)
        },
        |source, db, backup, report, rollback, denied| {
            validate_level_b_output_paths(source, db, backup, report, rollback, config, denied)
        },
        |source, db, failure| {
            apply_confirmed_workbench_state_root_to_confirmed_db(
                source,
                &config.confirmed_source_state_root,
                db,
                &config.confirmed_production_db_path,
                failure,
            )
        },
        |db, backup| export_verification_level_b(db, backup, &config.confirmed_production_db_path),
    )
}

fn rehearse_production_db_apply(
    level: &str,
    source_state_root: &Path,
    production_db_path: &Path,
    backup_root: &Path,
    report_path: &Path,
    rollback_manifest_path: &Path,
    config: &SqliteProductionApplyConfig,
    failure_point: Option<SqliteProductionApplyFailurePoint>,
    validate_source_root: impl Fn(&Path, &[String]) -> Result<(), String>,
    validate_output_paths: impl Fn(&Path, &Path, &Path, &Path, &Path, &[String]) -> Result<(), String>,
    apply_source_to_db: impl Fn(
        &Path,
        &Path,
        Option<SqliteApplyFailurePoint>,
    ) -> Result<SqliteApplyImportReport, String>,
    export_db: impl Fn(&Path, &Path) -> Result<SqliteProductionExportVerification, String>,
) -> Result<SqliteProductionApplyReport, String> {
    let denied_path_markers = effective_denied_path_markers(config);
    validate_source_root(source_state_root, &denied_path_markers)?;
    validate_output_paths(
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
    if level == LEVEL_B_WORKBENCH_OWNED_STATE {
        copy_source_files_to_backup(source_state_root, backup_root, &before_source_hashes)?;
    }
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
        level,
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
        let err = apply_source_to_db(source_state_root, production_db_path, None)
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
            apply_source_to_db,
            Some("corrupt_snapshot"),
        );
    }
    if failure_point == Some(SqliteProductionApplyFailurePoint::TransactionRollbackBeforeCommit) {
        let result = apply_source_to_db(
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

    let apply_report = apply_source_to_db(source_state_root, production_db_path, None)?;
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
        let export_verification = export_db(production_db_path, backup_root)?;
        let failed_report = SqliteProductionApplyReport {
            schema_version: PRODUCTION_APPLY_SCHEMA_VERSION.to_string(),
            mode: MODE.to_string(),
            level: level.to_string(),
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

    let mut export_verification = export_db(production_db_path, backup_root)?;
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
        level,
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
        level: level.to_string(),
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
    apply_source_to_db: impl Fn(
        &Path,
        &Path,
        Option<SqliteApplyFailurePoint>,
    ) -> Result<SqliteApplyImportReport, String>,
    label: Option<&str>,
) -> Result<T, String> {
    let err = apply_source_to_db(source_root, production_db_path, None)
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
        ..SqliteProductionPreflightConfig::default()
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
    level: &str,
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
        "level": level,
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

fn copy_source_files_to_backup(
    source_state_root: &Path,
    backup_root: &Path,
    source_hashes: &[SqliteProductionSourceFileHash],
) -> Result<(), String> {
    let backup_source_root = backup_root.join(BACKUP_SOURCE_DIR_NAME);
    fs::create_dir_all(&backup_source_root).map_err(|error| {
        format!(
            "create production apply source backup dir failed {}: {error}",
            backup_source_root.display()
        )
    })?;
    for source_file in source_hashes {
        if source_file.path_ref.contains('/') || source_file.path_ref.contains('\\') {
            return Err(format!(
                "production_apply_blocked:backup_source_path_must_be_flat:{}",
                source_file.path_ref
            ));
        }
        let source_path = source_state_root.join(&source_file.path_ref);
        let backup_path = backup_source_root.join(&source_file.path_ref);
        fs::copy(&source_path, &backup_path).map_err(|error| {
            format!(
                "copy production apply source backup failed {} -> {}: {error}",
                source_path.display(),
                backup_path.display()
            )
        })?;
        let copied = fs::read(&backup_path).map_err(|error| {
            format!(
                "read production apply source backup failed {}: {error}",
                backup_path.display()
            )
        })?;
        let copied_hash = sha256_hex(&copied);
        if copied_hash != source_file.file_hash {
            return Err(format!(
                "production_apply_blocked:backup_source_hash_mismatch:{}",
                source_file.path_ref
            ));
        }
    }
    Ok(())
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
    export_verification_from_manifest(manifest, true)
}

fn export_verification_level_b(
    production_db_path: &Path,
    backup_root: &Path,
    confirmed_production_db_path: &Path,
) -> Result<SqliteProductionExportVerification, String> {
    verify_db_integrity(production_db_path)?;
    let manifest = export_confirmed_db_to_json_dry_run(
        production_db_path,
        confirmed_production_db_path,
        &backup_root.display().to_string(),
    )?;
    export_verification_from_manifest(manifest, false)
}

fn export_verification_from_manifest(
    manifest: SqliteExportDryRunManifest,
    require_canonical_runtime_logs: bool,
) -> Result<SqliteProductionExportVerification, String> {
    verify_runtime_log_alias_policy(&manifest.projected_files, require_canonical_runtime_logs)?;
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
    level: &str,
) -> Result<String, String> {
    let manifest = json!({
        "schema_version": PRODUCTION_APPLY_SCHEMA_VERSION,
        "status": "completed",
        "mode": MODE,
        "level": level,
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

fn verify_runtime_log_alias_policy(
    files: &[SqliteProjectedFile],
    require_canonical_runtime_logs: bool,
) -> Result<(), String> {
    if files.iter().any(|file| file.path == "runtime-log.v1.json") {
        return Err("production_apply_blocked:legacy_runtime_log_alias_exported".to_string());
    }
    if require_canonical_runtime_logs
        && !files.iter().any(|file| file.path == "runtime-logs.v1.json")
    {
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

fn validate_level_b_source_root(
    path: &Path,
    confirmed_source_state_root: &Path,
    denied_path_markers: &[String],
) -> Result<(), String> {
    if path != confirmed_source_state_root {
        return Err(format!(
            "production_apply_level_b_source_root_mismatch:expected={}:actual={}",
            confirmed_source_state_root.display(),
            path.display()
        ));
    }
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!(
            "production_apply_source_root_required:{}",
            path.display()
        ));
    }
    let canonical_path = canonical_existing_dir(path, "source_root")?;
    let canonical_confirmed =
        canonical_existing_dir(confirmed_source_state_root, "confirmed_source_root")?;
    if canonical_path != canonical_confirmed || path != canonical_path.as_path() {
        return Err(format!(
            "production_apply_level_b_source_root_must_be_canonical:expected={}:actual={}",
            canonical_confirmed.display(),
            path.display()
        ));
    }
    if denied_path_hit(path, denied_path_markers) {
        return Err(format!(
            "production_apply_source_root_denied:{}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_level_b_output_paths(
    source_state_root: &Path,
    production_db_path: &Path,
    backup_root: &Path,
    report_path: &Path,
    rollback_manifest_path: &Path,
    config: &SqliteProductionApplyLevelBConfig,
    denied_path_markers: &[String],
) -> Result<(), String> {
    for (label, actual, expected) in [
        (
            "production_db_path",
            production_db_path,
            config.confirmed_production_db_path.as_path(),
        ),
        (
            "backup_root",
            backup_root,
            config.confirmed_backup_root.as_path(),
        ),
        (
            "report_path",
            report_path,
            config.confirmed_report_path.as_path(),
        ),
        (
            "rollback_manifest_path",
            rollback_manifest_path,
            config.confirmed_rollback_manifest_path.as_path(),
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "production_apply_level_b_confirmed_path_mismatch:{label}:expected={}:actual={}",
                expected.display(),
                actual.display()
            ));
        }
        if !actual.is_absolute() {
            return Err(format!(
                "production_apply_absolute_path_required:{}",
                actual.display()
            ));
        }
        require_clean_confirmed_output_path(label, actual)?;
        let canonical_actual = canonicalize_existing_or_parent(actual, label)?;
        let canonical_source_root = canonical_existing_dir(source_state_root, "source_root")?;
        if actual != canonical_actual.as_path() {
            return Err(format!(
                "production_apply_level_b_confirmed_path_must_be_canonical:{label}:expected={}:actual={}",
                canonical_actual.display(),
                actual.display()
            ));
        }
        if canonical_actual.starts_with(&canonical_source_root) {
            return Err(format!(
                "production_apply_blocked:path_inside_source_root_denied:{}",
                actual.display()
            ));
        }
        if denied_path_hit(actual, denied_path_markers) {
            return Err(format!(
                "production_apply_blocked:denied_path_marker:{}",
                actual.display()
            ));
        }
    }
    if report_path.starts_with(backup_root) || rollback_manifest_path.starts_with(source_state_root)
    {
        return Err("production_apply_report_or_rollback_path_invalid".to_string());
    }
    let canonical_backup = canonicalize_existing_or_parent(backup_root, "backup_root")?;
    let canonical_report = canonicalize_existing_or_parent(report_path, "report_path")?;
    let canonical_rollback =
        canonicalize_existing_or_parent(rollback_manifest_path, "rollback_manifest_path")?;
    let canonical_source = canonical_existing_dir(source_state_root, "source_root")?;
    if canonical_report.starts_with(&canonical_backup)
        || canonical_rollback.starts_with(&canonical_source)
    {
        return Err("production_apply_report_or_rollback_path_invalid".to_string());
    }
    Ok(())
}

fn require_clean_confirmed_output_path(label: &str, path: &Path) -> Result<(), String> {
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir | std::path::Component::CurDir
        )
    }) {
        return Err(format!(
            "production_apply_level_b_confirmed_path_must_be_clean:{label}:{}",
            path.display()
        ));
    }
    Ok(())
}

fn canonical_existing_dir(path: &Path, label: &str) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        format!(
            "production_apply_level_b_canonicalize_failed:{label}:{}:{error}",
            path.display()
        )
    })?;
    if !canonical.is_dir() {
        return Err(format!(
            "production_apply_level_b_canonical_dir_required:{label}:{}",
            path.display()
        ));
    }
    Ok(canonical)
}

fn canonicalize_existing_or_parent(path: &Path, label: &str) -> Result<PathBuf, String> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|error| {
            format!(
                "production_apply_level_b_canonicalize_failed:{label}:{}:{error}",
                path.display()
            )
        });
    }
    let mut ancestor = path.parent().ok_or_else(|| {
        format!(
            "production_apply_level_b_parent_required:{label}:{}",
            path.display()
        )
    })?;
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or_else(|| {
            format!(
                "production_apply_level_b_existing_parent_required:{label}:{}",
                path.display()
            )
        })?;
    }
    let canonical_ancestor = fs::canonicalize(ancestor).map_err(|error| {
        format!(
            "production_apply_level_b_parent_canonicalize_failed:{label}:{}:{error}",
            ancestor.display()
        )
    })?;
    let suffix = path.strip_prefix(ancestor).map_err(|error| {
        format!(
            "production_apply_level_b_suffix_failed:{label}:{}:{error}",
            path.display()
        )
    })?;
    Ok(canonical_ancestor.join(suffix))
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

    #[test]
    fn sqlite_production_level_b_accepts_confirmed_root_and_paths() {
        let source = copied_confirmed_source("level-b-success", false);
        let paths = prepare_level_b_paths("level-b-success");
        let config = level_b_config(&source, &paths);

        let report = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect("level b confirmed apply");

        assert_eq!(report.level, LEVEL_B_WORKBENCH_OWNED_STATE);
        assert_eq!(report.status, "completed");
        assert!(report.safety_flags.production_db_created);
        assert!(report.safety_flags.production_apply_performed);
        assert!(!report.safety_flags.read_cut_enabled);
        assert!(!report.safety_flags.stop_write_json);
        assert!(!report.safety_flags.source_json_written);
        assert_eq!(report.before_source_hashes, report.after_source_hashes);
        assert!(paths.db_path.exists());
        assert!(paths.backup_root.join(BACKUP_MANIFEST_NAME).exists());
        assert!(paths
            .backup_root
            .join(BACKUP_SOURCE_DIR_NAME)
            .join(PRIMARY_WORKFLOW_STATE)
            .exists());
        assert!(paths.rollback_manifest_path.exists());
    }

    #[test]
    fn sqlite_production_level_b_missing_optional_sidecars_warns_not_fails() {
        let source = copied_confirmed_source("level-b-sparse-sidecars", true);
        let paths = prepare_level_b_paths("level-b-sparse-sidecars");
        let config = level_b_config(&source, &paths);

        let report = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect("sparse sidecar source should still apply");

        assert_eq!(report.level, LEVEL_B_WORKBENCH_OWNED_STATE);
        assert_eq!(report.status, "completed");
        assert_eq!(report.source_record_counts.len(), 1);
        assert!(report
            .source_record_counts
            .contains_key(PRIMARY_WORKFLOW_STATE));
        assert_eq!(report.before_source_hashes, report.after_source_hashes);
    }

    #[test]
    fn sqlite_production_level_b_rejects_unconfirmed_source_root() {
        let source = copied_confirmed_source("level-b-source", false);
        let other_source = copied_confirmed_source("level-b-other-source", false);
        let paths = prepare_level_b_paths("level-b-source-mismatch");
        let config = level_b_config(&source, &paths);

        let err = rehearse_production_db_apply_level_b_workbench_owned_state(
            &other_source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect_err("unconfirmed source root must reject");

        assert!(err.contains("source_root_mismatch"));
        assert!(!paths.db_path.exists());
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_production_level_b_rejects_unconfirmed_output_paths() {
        let source = copied_confirmed_source("level-b-output", false);
        let paths = prepare_level_b_paths("level-b-output");
        let config = level_b_config(&source, &paths);

        for (db_path, backup_root, report_path, rollback_path, expected) in [
            (
                paths.db_path.with_file_name("other.sqlite"),
                paths.backup_root.clone(),
                paths.report_path.clone(),
                paths.rollback_manifest_path.clone(),
                "production_db_path",
            ),
            (
                paths.db_path.clone(),
                paths.backup_root.with_file_name("other-backup"),
                paths.report_path.clone(),
                paths.rollback_manifest_path.clone(),
                "backup_root",
            ),
            (
                paths.db_path.clone(),
                paths.backup_root.clone(),
                paths.report_path.with_file_name("other-report.json"),
                paths.rollback_manifest_path.clone(),
                "report_path",
            ),
            (
                paths.db_path.clone(),
                paths.backup_root.clone(),
                paths.report_path.clone(),
                paths
                    .rollback_manifest_path
                    .with_file_name("other-rollback.json"),
                "rollback_manifest_path",
            ),
        ] {
            let err = rehearse_production_db_apply_level_b_workbench_owned_state(
                &source,
                &db_path,
                &backup_root,
                &report_path,
                &rollback_path,
                &config,
                None,
            )
            .expect_err("unconfirmed output path must reject");
            assert!(err.contains("confirmed_path_mismatch"));
            assert!(err.contains(expected));
        }
    }

    #[test]
    fn sqlite_production_level_b_rejects_denied_output_path_marker() {
        let source = copied_confirmed_source("level-b-denied-output", false);
        let temp_dir = fs::canonicalize(std::env::temp_dir()).expect("canonical temp dir");
        let root = temp_dir.join(unique_label("workbench-r3-b1-secret"));
        let paths = Paths {
            db_path: root.join("secret").join("production.sqlite"),
            backup_root: root.join("backup"),
            report_path: root.join("reports").join("production-apply-report.json"),
            rollback_manifest_path: root.join("rollback").join("rollback-manifest.json"),
        };
        let config = level_b_config(&source, &paths);

        let err = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect_err("denied marker must reject");

        assert!(err.contains("denied_path_marker"));
    }

    #[test]
    fn sqlite_production_level_b_rejects_output_inside_source_root() {
        let source = copied_confirmed_source("level-b-inside-source", false);
        let paths = Paths {
            db_path: source.join("production.sqlite"),
            backup_root: std::env::temp_dir().join(unique_label("level-b-backup")),
            report_path: std::env::temp_dir()
                .join(unique_label("level-b-report"))
                .join("report.json"),
            rollback_manifest_path: std::env::temp_dir()
                .join(unique_label("level-b-rollback"))
                .join("rollback.json"),
        };
        let config = level_b_config(&source, &paths);

        let err = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect_err("inside source output must reject");

        assert!(err.contains("path_inside_source_root_denied"));
        assert!(!paths.db_path.exists());
    }

    #[test]
    fn sqlite_production_level_b_rejects_parent_dir_output_escape_into_source_root() {
        let source = copied_confirmed_source("level-b-parent-dir-source", false);
        let paths = prepare_level_b_paths("level-b-parent-dir-source");
        let db_path = source
            .join("outside")
            .join("..")
            .join("production-via-parent.sqlite");
        let paths = Paths { db_path, ..paths };
        let config = level_b_config(&source, &paths);

        let err = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect_err("parent-dir output into source root must reject");

        assert!(err.contains("confirmed_path_must_be_clean"));
        assert!(!source.join("production-via-parent.sqlite").exists());
    }

    #[test]
    fn sqlite_production_level_a_still_rejects_non_temp_output_path() {
        let source = fixture_dir("production-valid-core-chain");
        let paths = Paths {
            db_path: PathBuf::from("/var/workbench-r3-b1-enable.sqlite"),
            backup_root: PathBuf::from("/var/workbench-r3-b1-backup"),
            report_path: PathBuf::from("/var/workbench-r3-b1-report.json"),
            rollback_manifest_path: PathBuf::from("/var/workbench-r3-b1-rollback.json"),
        };

        let err = rehearse_production_db_apply_level_a(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &SqliteProductionApplyConfig::default(),
            None,
        )
        .expect_err("level a must keep temp path guard");

        assert!(err.contains("production_apply_temp_paths_required"));
    }

    #[test]
    #[ignore = "requires explicit R3 B1 production apply authorization and confirmed paths"]
    fn r3_b1_production_apply_confirmed_paths_requires_env_authorization() {
        let confirmation = std::env::var("R3_B1_APPLY_CONFIRM")
            .expect("R3_B1_APPLY_CONFIRM is required for real B1 apply");
        assert_eq!(confirmation, "CONFIRMED_USER_PRESENT_2026_06_15");
        let expected_source_hash = std::env::var("R3_B1_EXPECTED_SOURCE_ROOT_HASH")
            .expect("R3_B1_EXPECTED_SOURCE_ROOT_HASH is required for real B1 apply");
        let source = canonical_env_path("R3_B1_SOURCE_STATE_ROOT");
        let paths = Paths {
            db_path: canonical_parent_env_path("R3_B1_PRODUCTION_DB_PATH"),
            backup_root: canonical_parent_env_path("R3_B1_BACKUP_ROOT"),
            report_path: canonical_parent_env_path("R3_B1_REPORT_PATH"),
            rollback_manifest_path: canonical_parent_env_path("R3_B1_ROLLBACK_MANIFEST_PATH"),
        };
        let config = SqliteProductionApplyLevelBConfig {
            apply_config: SqliteProductionApplyConfig {
                expected_source_root_hash: Some(expected_source_hash),
                ..SqliteProductionApplyConfig::default()
            },
            confirmed_source_state_root: source.clone(),
            confirmed_production_db_path: paths.db_path.clone(),
            confirmed_backup_root: paths.backup_root.clone(),
            confirmed_report_path: paths.report_path.clone(),
            confirmed_rollback_manifest_path: paths.rollback_manifest_path.clone(),
        };

        let report = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &paths.db_path,
            &paths.backup_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &config,
            None,
        )
        .expect("R3 B1 real production apply must complete");

        assert_eq!(report.level, LEVEL_B_WORKBENCH_OWNED_STATE);
        assert_eq!(report.status, "completed");
        assert!(report.safety_flags.production_db_created);
        assert!(report.safety_flags.production_apply_performed);
        assert!(!report.safety_flags.read_cut_enabled);
        assert!(!report.safety_flags.stop_write_json);
        assert!(!report.safety_flags.source_json_written);
        assert!(!report.safety_flags.codex_home_touched);
        assert_eq!(report.before_source_hashes, report.after_source_hashes);
        assert!(paths.db_path.exists());
        assert!(paths.backup_root.join(BACKUP_MANIFEST_NAME).exists());
        assert!(paths.backup_root.join(APPLY_MANIFEST_NAME).exists());
        assert!(paths.backup_root.join(EXPORT_MANIFEST_NAME).exists());
        assert!(paths.rollback_manifest_path.exists());
        assert!(paths.report_path.exists());
        println!(
            "R3_B1_PRODUCTION_APPLY_REPORT_PATH={}",
            paths.report_path.display()
        );
        println!("R3_B1_PRODUCTION_DB_PATH={}", paths.db_path.display());
        println!(
            "R3_B1_BACKUP_MANIFEST_PATH={}",
            paths.backup_root.join(BACKUP_MANIFEST_NAME).display()
        );
        println!(
            "R3_B1_ROLLBACK_MANIFEST_PATH={}",
            paths.rollback_manifest_path.display()
        );
        println!("R3_B1_SOURCE_ROOT_HASH={}", report.source_root_hash);
        println!(
            "R3_B1_EXPORT_HASH={}",
            report.export_verification.db_export_hash
        );
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
        prepare_paths_under_root(root)
    }

    fn prepare_level_b_paths(label: &str) -> Paths {
        let unique = unique_label(label);
        let temp_dir = fs::canonicalize(std::env::temp_dir()).expect("canonical temp dir");
        let root = temp_dir.join(format!("workbench-r3-a9-{unique}"));
        prepare_paths_under_root(root)
    }

    fn prepare_paths_under_root(root: PathBuf) -> Paths {
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root");
        Paths {
            db_path: root.join("production.sqlite"),
            backup_root: root.join("backup"),
            report_path: root.join("reports").join("production-apply-report.json"),
            rollback_manifest_path: root.join("rollback").join("rollback-manifest.json"),
        }
    }

    fn level_b_config(source: &Path, paths: &Paths) -> SqliteProductionApplyLevelBConfig {
        SqliteProductionApplyLevelBConfig {
            apply_config: SqliteProductionApplyConfig::default(),
            confirmed_source_state_root: source.to_path_buf(),
            confirmed_production_db_path: paths.db_path.clone(),
            confirmed_backup_root: paths.backup_root.clone(),
            confirmed_report_path: paths.report_path.clone(),
            confirmed_rollback_manifest_path: paths.rollback_manifest_path.clone(),
        }
    }

    fn copied_confirmed_source(label: &str, sparse_sidecars: bool) -> PathBuf {
        let temp_dir = fs::canonicalize(std::env::temp_dir()).expect("canonical temp dir");
        let root = temp_dir.join(format!(
            "workbench-r3-b1-confirmed-source-{}",
            unique_label(label)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("confirmed source root");
        for entry in
            fs::read_dir(fixture_dir("production-valid-core-chain")).expect("read source fixture")
        {
            let entry = entry.expect("source fixture entry");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name();
            let name_text = name.to_string_lossy();
            if sparse_sidecars && name_text.as_ref() != PRIMARY_WORKFLOW_STATE {
                continue;
            }
            fs::copy(&path, root.join(name)).expect("copy confirmed source file");
        }
        root
    }

    fn canonical_env_path(name: &str) -> PathBuf {
        let value = std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"));
        fs::canonicalize(&value)
            .unwrap_or_else(|error| panic!("canonicalize {name} failed for {value}: {error}"))
    }

    fn canonical_parent_env_path(name: &str) -> PathBuf {
        let value = std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"));
        let path = PathBuf::from(&value);
        if path.exists() {
            return fs::canonicalize(&path).unwrap_or_else(|error| {
                panic!("canonicalize existing {name} failed for {value}: {error}")
            });
        }
        let parent = path
            .parent()
            .unwrap_or_else(|| panic!("{name} has no parent: {value}"));
        let mut ancestor = parent;
        while !ancestor.exists() {
            ancestor = ancestor
                .parent()
                .unwrap_or_else(|| panic!("{name} has no existing ancestor: {value}"));
        }
        let canonical_ancestor = fs::canonicalize(ancestor).unwrap_or_else(|error| {
            panic!(
                "canonicalize existing ancestor for {name} failed {}: {error}",
                ancestor.display()
            )
        });
        let suffix = path
            .strip_prefix(ancestor)
            .unwrap_or_else(|error| panic!("strip prefix for {name} failed: {error}"));
        canonical_ancestor.join(suffix)
    }

    fn unique_label(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        format!("{label}-{nanos}")
    }

    // M3 (2026-07-13): live-snapshot rehearsal. Ignored by default — reads the user's real
    // workflow-state root (READ-ONLY) via WORKBENCH_M3_LIVE_ROOT, stages a pruned canonical temp
    // copy, runs the real Level-B cutover pipeline against it, reconciles the JSON→SQLite→JSON
    // round-trip, exercises idempotence + mid-transaction crash rollback, and proves the live root
    // is byte-for-byte unchanged. Run: WORKBENCH_M3_LIVE_ROOT="<root>" cargo test --lib \
    //   sqlite_m3_live_snapshot -- --ignored --nocapture
    #[test]
    #[ignore = "M3 live-snapshot rehearsal: needs WORKBENCH_M3_LIVE_ROOT (reads real live data)"]
    fn sqlite_m3_live_snapshot_level_b_rehearsal_and_reconcile() {
        use crate::workbench_sqlite_exporter::export_confirmed_db_to_json_dry_run;
        use crate::workbench_sqlite_importer::dry_run_import_fixture_dir;
        let live_root = PathBuf::from(
            std::env::var("WORKBENCH_M3_LIVE_ROOT")
                .expect("set WORKBENCH_M3_LIVE_ROOT to the live workflow-state root"),
        );
        assert!(
            live_root.is_dir(),
            "live root must be a dir: {}",
            live_root.display()
        );

        // 1) Hash the ENTIRE live tree BEFORE touching anything (red-line proof).
        let live_hash_before = hash_dir_tree(&live_root);

        // 2) Stage a pruned canonical temp copy: primary + whitelisted sidecars that exist.
        //    (Drops the 91 supervisor:*.txt, exec-process-registry.v1.json, backups/ — all correctly
        //    excluded since they are not PRIMARY nor in OPTIONAL_SIDECARS.)
        let temp_dir = fs::canonicalize(std::env::temp_dir()).expect("canonical temp dir");
        let copy_root = temp_dir.join(format!("workbench-m3-live-{}", unique_label("copy")));
        let _ = fs::remove_dir_all(&copy_root);
        fs::create_dir_all(&copy_root).expect("copy root");
        let mut staged = Vec::new();
        for name in std::iter::once(PRIMARY_WORKFLOW_STATE).chain(OPTIONAL_SIDECARS.iter().copied())
        {
            let src = live_root.join(name);
            if src.is_file() {
                fs::copy(&src, copy_root.join(name)).expect("copy staged file");
                staged.push(name.to_string());
            }
        }
        assert!(
            staged.iter().any(|n| n == PRIMARY_WORKFLOW_STATE),
            "primary workflow-state.v0.json must be present in live root"
        );
        eprintln!("[M3] staged {} files: {staged:?}", staged.len());

        // 3) Dry-run against the real pruned snapshot. This is the crux of the rehearsal:
        //    it exercises the importer's gates against real data before any switch-flip.
        let dry = dry_run_import_fixture_dir(&copy_root).expect("dry run");
        let inventory: Vec<(String, String)> = dry
            .source_inventory
            .iter()
            .map(|s| (s.source_kind.clone(), s.classification.clone()))
            .collect();
        eprintln!(
            "[M3] dry_run batch_status={} conflicts={} inventory={inventory:?}",
            dry.batch_status, dry.counts.conflicts
        );

        // The staged snapshot must pass the importer gate before the full Level-B reconciliation.
        assert!(
            dry.batch_status == "accepted" || dry.batch_status == "accepted_with_rejections",
            "pruned live copy not applyable: {}",
            dry.batch_status
        );
        assert_eq!(
            dry.counts.conflicts, 0,
            "unexpected conflicts on live snapshot"
        );

        let out = temp_dir.join(format!("workbench-m3-out-{}", unique_label("out")));
        let _ = fs::remove_dir_all(&out);
        fs::create_dir_all(&out).expect("out root");
        let db_path = out.join("production.sqlite");
        let backup_root = out.join("backup");
        let report_path = out.join("reports").join("m3-report.json");
        let rollback_path = out.join("rollback").join("m3-rollback.json");
        let config = SqliteProductionApplyLevelBConfig {
            apply_config: SqliteProductionApplyConfig::default(),
            confirmed_source_state_root: copy_root.clone(),
            confirmed_production_db_path: db_path.clone(),
            confirmed_backup_root: backup_root.clone(),
            confirmed_report_path: report_path.clone(),
            confirmed_rollback_manifest_path: rollback_path.clone(),
        };
        let report = rehearse_production_db_apply_level_b_workbench_owned_state(
            &copy_root,
            &db_path,
            &backup_root,
            &report_path,
            &rollback_path,
            &config,
            None,
        )
        .expect("level b rehearse");
        assert_eq!(report.status, "completed");
        assert!(!report.safety_flags.source_json_written);
        let primary: Value =
            serde_json::from_slice(&fs::read(copy_root.join(PRIMARY_WORKFLOW_STATE)).unwrap())
                .unwrap();
        for array in [
            "execution_attempts",
            "permission_requests",
            "workflow_chain_runs",
            "workflow_execution_controls",
            "workflow_machine_runs",
        ] {
            let src = primary
                .get(array)
                .and_then(Value::as_array)
                .map_or(0, Vec::len) as i64;
            assert_eq!(src, table_count(&db_path, array).expect("count"), "{array}");
            eprintln!("[M3] {array}: source={src} reconciled");
        }
        let manifest = export_confirmed_db_to_json_dry_run(&db_path, &db_path, "m3")
            .expect("export confirmed");
        let exp_rev = manifest
            .projected_files
            .iter()
            .find(|f| f.path == "workflow-state.v0.json")
            .and_then(|f| f.projection.get("revision").and_then(Value::as_i64));
        assert_eq!(primary.get("revision").and_then(Value::as_i64), exp_rev);
        let live_hash_after = hash_dir_tree(&live_root);
        assert_eq!(live_hash_before, live_hash_after, "live root modified");
        eprintln!("[M3] full live round-trip reconciled; live root untouched");
    }

    // M3 mechanism proof at representative scale (runs in the normal suite). Builds a synthetic
    // primary carrying the five previously-dropped main-store arrays + revision=11 (matching the
    // live main store), runs the real Level-B cutover pipeline, reconciles the JSON→SQLite→JSON
    // round-trip + revision fidelity, and exercises idempotence and a mid-transaction crash rollback.
    #[test]
    fn sqlite_m3_synthetic_live_scale_level_b_round_trip() {
        use crate::workbench_sqlite_exporter::export_confirmed_db_to_json_dry_run;
        let temp_dir = fs::canonicalize(std::env::temp_dir()).expect("canonical temp dir");
        let source = temp_dir.join(format!("workbench-m3-syn-{}", unique_label("src")));
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&source).expect("source dir");

        let attempts: Vec<Value> = (0..24)
            .map(|i| serde_json::json!({"attempt_id": format!("attempt-{i}"), "workflow_id": "wf-1", "work_item_id": format!("wi-{i}"), "state": "succeeded"}))
            .collect();
        let controls: Vec<Value> = (0..24)
            .map(|i| serde_json::json!({"control_id": format!("ctrl-{i}"), "workflow_id": "wf-1", "control_state": "active"}))
            .collect();
        let chains: Vec<Value> = (0..7)
            .map(|i| serde_json::json!({"chain_run_id": format!("chain-{i}"), "workflow_id": "wf-1", "state": "ended", "nodes": [{"node_id": "n1"}]}))
            .collect();
        let machines: Vec<Value> = (0..3)
            .map(|i| serde_json::json!({"run_id": format!("machine-{i}"), "workflow_id": "wf-1", "state": "ended"}))
            .collect();
        let state = serde_json::json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "revision": 11,
            "projects": [], "agent_adapters": [], "workflows": [], "nodes": [], "edges": [],
            "work_items": [], "artifacts": [], "reviews": [], "audit_events": [],
            "capabilities": [], "harness_resources": [],
            "workflow_node_session_bindings": [], "workflow_node_dispatches": [],
            "execution_attempts": attempts,
            "permission_requests": [{"request_id": "req-1", "workflow_id": "wf-1", "status": "pending"}],
            "workflow_chain_runs": chains,
            "workflow_execution_controls": controls,
            "workflow_machine_runs": machines
        });
        fs::write(
            source.join(PRIMARY_WORKFLOW_STATE),
            serde_json::to_vec_pretty(&state).expect("serialize"),
        )
        .expect("write primary");

        let out = temp_dir.join(format!("workbench-m3-syn-out-{}", unique_label("out")));
        let _ = fs::remove_dir_all(&out);
        fs::create_dir_all(&out).expect("out");
        let db_path = out.join("production.sqlite");
        let config = SqliteProductionApplyLevelBConfig {
            apply_config: SqliteProductionApplyConfig::default(),
            confirmed_source_state_root: source.clone(),
            confirmed_production_db_path: db_path.clone(),
            confirmed_backup_root: out.join("backup"),
            confirmed_report_path: out.join("reports").join("r.json"),
            confirmed_rollback_manifest_path: out.join("rollback").join("rb.json"),
        };
        let report = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &db_path,
            &out.join("backup"),
            &out.join("reports").join("r.json"),
            &out.join("rollback").join("rb.json"),
            &config,
            None,
        )
        .expect("level b synthetic");
        assert_eq!(report.status, "completed");
        assert!(!report.safety_flags.source_json_written);
        assert_eq!(report.before_source_hashes, report.after_source_hashes);
        assert_eq!(table_count(&db_path, "execution_attempts").unwrap(), 24);
        assert_eq!(table_count(&db_path, "permission_requests").unwrap(), 1);
        assert_eq!(table_count(&db_path, "workflow_chain_runs").unwrap(), 7);
        assert_eq!(
            table_count(&db_path, "workflow_execution_controls").unwrap(),
            24
        );
        assert_eq!(table_count(&db_path, "workflow_machine_runs").unwrap(), 3);

        // Revision fidelity: 11 round-trips (not defaulted to 1).
        let manifest =
            export_confirmed_db_to_json_dry_run(&db_path, &db_path, "m3-syn").expect("export");
        let exp_rev = manifest
            .projected_files
            .iter()
            .find(|f| f.path == "workflow-state.v0.json")
            .and_then(|f| f.projection.get("revision").and_then(Value::as_i64));
        assert_eq!(
            exp_rev,
            Some(11),
            "revision 11 must round-trip, not default to 1"
        );

        // Idempotence (apply_fixture_dir_to_temp_db gates the DB on the NON-canonical temp dir).
        let idem_db = std::env::temp_dir().join(format!(
            "workbench-m3-syn-idem-{}.sqlite",
            unique_label("idem")
        ));
        apply_fixture_dir_to_temp_db(&source, &idem_db, None).expect("idem 1");
        apply_fixture_dir_to_temp_db(&source, &idem_db, None).expect("idem 2");
        assert_eq!(table_count(&idem_db, "execution_attempts").unwrap(), 24);

        // Mid-transaction crash rolls back atomically.
        let fail_out = temp_dir.join(format!("workbench-m3-syn-fail-{}", unique_label("f")));
        fs::create_dir_all(&fail_out).expect("fail out");
        let fail_db = fail_out.join("production.sqlite");
        let fail_config = SqliteProductionApplyLevelBConfig {
            apply_config: SqliteProductionApplyConfig::default(),
            confirmed_source_state_root: source.clone(),
            confirmed_production_db_path: fail_db.clone(),
            confirmed_backup_root: fail_out.join("backup"),
            confirmed_report_path: fail_out.join("r.json"),
            confirmed_rollback_manifest_path: fail_out.join("rb.json"),
        };
        let injected = rehearse_production_db_apply_level_b_workbench_owned_state(
            &source,
            &fail_db,
            &fail_out.join("backup"),
            &fail_out.join("r.json"),
            &fail_out.join("rb.json"),
            &fail_config,
            Some(SqliteProductionApplyFailurePoint::TransactionRollbackBeforeCommit),
        );
        assert!(injected.is_err(), "mid-transaction crash must fail closed");
    }

    fn hash_dir_tree(root: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut entries: Vec<(String, String)> = Vec::new();
        collect_file_hashes(root, root, &mut entries);
        entries.sort();
        let mut hasher = Sha256::new();
        for (rel, hash) in entries {
            hasher.update(rel.as_bytes());
            hasher.update(b"=");
            hasher.update(hash.as_bytes());
            hasher.update(b"\n");
        }
        format!("{:x}", hasher.finalize())
    }

    fn collect_file_hashes(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        use sha2::{Digest, Sha256};
        let Ok(read) = fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_file_hashes(base, &path, out);
            } else if let Ok(bytes) = fs::read(&path) {
                let mut hasher = Sha256::new();
                hasher.update(&bytes);
                let rel = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                out.push((rel, format!("{:x}", hasher.finalize())));
            }
        }
    }
}
