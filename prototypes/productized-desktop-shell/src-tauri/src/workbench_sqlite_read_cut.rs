use crate::utils::hash::sha256_hex_bytes as sha256_hex;
use crate::workbench_sqlite_apply::{apply_fixture_dir_to_temp_db, table_count};
use crate::workbench_sqlite_exporter::{export_temp_db_to_json_dry_run, SqliteProjectedFile};
use crate::workbench_sqlite_importer::canonical_json_hash;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const LEVEL_A: &str = "level_a_fixture";
const LIMITED_READ_CUT_MODE: &str = "limited_read_cut";
const LIMITED_READ_CUT_SCHEMA_VERSION: &str = "workbench_sqlite_limited_read_cut.v1";
const WORKFLOW_STATE_SUMMARY_READ_MODEL: &str = "workflow_state_summary";
const DEFAULT_LIMITED_READ_CUT_DENIED_PATH_MARKERS: &[&str] = &[
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
pub(crate) enum SqliteReadCutFailurePoint {
    BeforeDbRead,
    AfterDbReadBeforeProjectionVerification,
    ProjectionHashMismatch,
    MissingRollbackManifest,
    IncompleteRollbackManifest,
    DbUnavailable,
    CorruptDbPathOrSchemaMismatch,
    AfterFallbackSelectedBeforeReportCommit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteReadCutRehearsalReport {
    pub(crate) status: String,
    pub(crate) read_source: String,
    pub(crate) fallback_decision: String,
    pub(crate) degraded: bool,
    pub(crate) source_root_hash: String,
    pub(crate) db_path_ref: String,
    pub(crate) projection_root_ref: String,
    pub(crate) manifest_path_ref: String,
    pub(crate) read_cut_report_path_ref: String,
    pub(crate) db_read_hash: Option<String>,
    pub(crate) projection_hash: Option<String>,
    pub(crate) source_projection_hash: Option<String>,
    pub(crate) projected_files: Vec<SqliteReadCutProjectedFile>,
    pub(crate) db_row_counts: BTreeMap<String, i64>,
    pub(crate) rollback_manifest_hash: String,
    pub(crate) recovery_dry_run: SqliteReadCutRecoveryDryRun,
    pub(crate) failure_point: Option<String>,
    pub(crate) redaction_policy: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteReadCutProjectedFile {
    pub(crate) path: String,
    pub(crate) projected_hash: String,
    pub(crate) record_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteReadCutRecoveryDryRun {
    pub(crate) status: String,
    pub(crate) would_use_db: bool,
    pub(crate) would_use_json_projection: bool,
    pub(crate) would_disable_db_read_cut: bool,
    pub(crate) would_preserve_db_for_audit: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) instructions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteLimitedReadCutFailurePoint {
    DbUnavailable,
    DbSchemaMismatch,
    DbIntegrityFailure,
    DbHashMismatch,
    FallbackHashMismatch,
    ProjectionHashMismatch,
    MissingRollbackManifest,
    IncompleteRollbackManifest,
    AfterDbReadBeforeReportCommit,
    AfterFallbackSelectedBeforeReportCommit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteLimitedReadCutReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) level: String,
    pub(crate) status: String,
    pub(crate) read_model_name: String,
    pub(crate) feature_flag_enabled: bool,
    pub(crate) read_source: String,
    pub(crate) fallback_decision: String,
    pub(crate) degraded: bool,
    pub(crate) db_path_hash: String,
    pub(crate) fallback_root_hash: String,
    pub(crate) projection_hash: String,
    pub(crate) rollback_manifest_hash: String,
    pub(crate) expected_db_hash: Option<String>,
    pub(crate) actual_db_hash: Option<String>,
    pub(crate) expected_fallback_hash: Option<String>,
    pub(crate) actual_fallback_hash: String,
    pub(crate) counts: BTreeMap<String, usize>,
    pub(crate) recovery_dry_run: SqliteLimitedReadCutRecoveryDryRun,
    pub(crate) safety_flags: SqliteLimitedReadCutSafetyFlags,
    pub(crate) failure_point: Option<String>,
    pub(crate) redaction_policy: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteLimitedReadCutRecoveryDryRun {
    pub(crate) status: String,
    pub(crate) would_disable_limited_read_cut: bool,
    pub(crate) would_use_json_fallback: bool,
    pub(crate) would_preserve_db_for_audit: bool,
    pub(crate) would_require_supervisor_decision: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) instructions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteLimitedReadCutSafetyFlags {
    pub(crate) limited_read_cut_enabled: bool,
    pub(crate) product_global_read_path_changed: bool,
    pub(crate) app_startup_reads_db: bool,
    pub(crate) tauri_command_reads_db: bool,
    pub(crate) ui_reads_db: bool,
    pub(crate) stop_write_json: bool,
    pub(crate) source_json_written: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) codex_home_touched: bool,
}

impl SqliteLimitedReadCutSafetyFlags {
    fn for_db_success() -> Self {
        Self {
            limited_read_cut_enabled: true,
            product_global_read_path_changed: false,
            app_startup_reads_db: false,
            tauri_command_reads_db: false,
            ui_reads_db: false,
            stop_write_json: false,
            source_json_written: false,
            production_restore_performed: false,
            codex_home_touched: false,
        }
    }

    fn for_fallback() -> Self {
        Self {
            limited_read_cut_enabled: false,
            product_global_read_path_changed: false,
            app_startup_reads_db: false,
            tauri_command_reads_db: false,
            ui_reads_db: false,
            stop_write_json: false,
            source_json_written: false,
            production_restore_performed: false,
            codex_home_touched: false,
        }
    }
}

pub(crate) fn rehearse_limited_read_cut_level_a(
    read_model_name: &str,
    feature_flag_enabled: bool,
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    read_cut_report_path: &Path,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    allowed_read_models: &BTreeSet<String>,
    denied_path_markers: &[String],
    failure_point: Option<SqliteLimitedReadCutFailurePoint>,
) -> Result<SqliteLimitedReadCutReport, String> {
    validate_limited_read_model(read_model_name, allowed_read_models)?;
    let denied_markers = effective_limited_denied_markers(denied_path_markers);
    validate_limited_read_cut_paths(
        db_path,
        json_fallback_root,
        projection_root,
        rollback_manifest_path,
        read_cut_report_path,
        &denied_markers,
    )?;
    remove_file_if_exists(read_cut_report_path)?;

    let fallback = verified_limited_json_fallback(
        json_fallback_root,
        read_model_name,
        expected_fallback_hash,
        &denied_markers,
        failure_point,
    )?;

    if !feature_flag_enabled {
        let report = limited_read_cut_report(
            "feature_flag_disabled_fallback",
            read_model_name,
            false,
            "json_fallback",
            "feature_flag_disabled",
            true,
            db_path,
            &fallback.fallback_root_hash,
            &fallback.projection_hash,
            &fallback.manifest_hash,
            expected_db_hash,
            None,
            expected_fallback_hash,
            &fallback.projection_hash,
            fallback.counts,
            SqliteLimitedReadCutSafetyFlags::for_fallback(),
            failure_point,
        );
        write_limited_read_cut_report(read_cut_report_path, &report)?;
        return Ok(report);
    }

    let mut actual_db_hash = if db_path.exists() {
        Some(file_hash(db_path)?)
    } else {
        None
    };
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::DbHashMismatch) {
        actual_db_hash = Some("injected_db_hash_mismatch".to_string());
    }
    if let Some(expected) = expected_db_hash {
        if actual_db_hash.as_deref() != Some(expected) {
            remove_file_if_exists(read_cut_report_path)?;
            return Err(format!(
                "limited_read_cut_blocked:db_hash_mismatch:expected={expected}:actual={}",
                actual_db_hash.unwrap_or_else(|| "missing".to_string())
            ));
        }
    }

    if failure_point == Some(SqliteLimitedReadCutFailurePoint::DbUnavailable) {
        remove_file_if_exists(db_path)?;
    }
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::DbSchemaMismatch) {
        remove_file_if_exists(db_path)?;
        let connection = Connection::open(db_path).map_err(|error| {
            format!(
                "create limited read-cut schema mismatch db failed {}: {error}",
                db_path.display()
            )
        })?;
        connection
            .execute_batch("CREATE TABLE wrong_schema_marker (id TEXT PRIMARY KEY);")
            .map_err(|error| {
                format!(
                    "write limited read-cut schema mismatch db failed {}: {error}",
                    db_path.display()
                )
            })?;
    }
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::DbIntegrityFailure) {
        fs::write(db_path, b"not a sqlite database").map_err(|error| {
            format!(
                "write corrupt limited read-cut db fixture failed {}: {error}",
                db_path.display()
            )
        })?;
    }

    match read_limited_db_model(db_path, read_model_name, projection_root, failure_point) {
        Ok(db_projection) => {
            if failure_point
                == Some(SqliteLimitedReadCutFailurePoint::AfterDbReadBeforeReportCommit)
            {
                remove_file_if_exists(read_cut_report_path)?;
                return Err("injected_failure_after_db_read_before_report_commit".to_string());
            }
            let manifest_hash =
                write_limited_rollback_manifest(rollback_manifest_path, &db_projection)?;
            let verified_manifest_hash = verify_limited_rollback_manifest(
                rollback_manifest_path,
                failure_point,
                "limited_read_cut_blocked",
            )?;
            if manifest_hash != verified_manifest_hash {
                return Err("limited_read_cut_blocked:rollback_manifest_hash_mismatch".to_string());
            }
            let report = limited_read_cut_report(
                "completed",
                read_model_name,
                true,
                "db_limited",
                "not_used",
                false,
                db_path,
                &fallback.fallback_root_hash,
                &db_projection.projection_hash,
                &verified_manifest_hash,
                expected_db_hash,
                actual_db_hash.as_deref(),
                expected_fallback_hash,
                &fallback.projection_hash,
                db_projection.counts,
                SqliteLimitedReadCutSafetyFlags::for_db_success(),
                failure_point,
            );
            write_limited_read_cut_report(read_cut_report_path, &report)?;
            Ok(report)
        }
        Err(reason) if reason.starts_with("limited_read_cut_fallback:") => {
            if failure_point
                == Some(SqliteLimitedReadCutFailurePoint::AfterFallbackSelectedBeforeReportCommit)
            {
                remove_file_if_exists(read_cut_report_path)?;
                return Err(
                    "injected_failure_after_fallback_selected_before_report_commit".to_string(),
                );
            }
            let report = limited_read_cut_report(
                "fallback_degraded",
                read_model_name,
                true,
                "json_fallback",
                &format!("selected:{reason}"),
                true,
                db_path,
                &fallback.fallback_root_hash,
                &fallback.projection_hash,
                &fallback.manifest_hash,
                expected_db_hash,
                actual_db_hash.as_deref(),
                expected_fallback_hash,
                &fallback.projection_hash,
                fallback.counts,
                SqliteLimitedReadCutSafetyFlags::for_fallback(),
                failure_point,
            );
            write_limited_read_cut_report(read_cut_report_path, &report)?;
            Ok(report)
        }
        Err(reason) => {
            remove_file_if_exists(read_cut_report_path)?;
            Err(reason)
        }
    }
}

pub(crate) fn rehearse_fixture_read_cut(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    manifest_path: &Path,
    read_cut_report_path: &Path,
    failure_point: Option<SqliteReadCutFailurePoint>,
) -> Result<SqliteReadCutRehearsalReport, String> {
    validate_fixture_root(fixture_root)?;
    validate_temp_db_path(db_path)?;
    validate_projection_paths(projection_root, manifest_path, read_cut_report_path)?;
    remove_file_if_exists(read_cut_report_path)?;

    if failure_point == Some(SqliteReadCutFailurePoint::BeforeDbRead) {
        return Err("injected_failure_before_db_read".to_string());
    }

    let apply_report = apply_fixture_dir_to_temp_db(fixture_root, db_path, None)?;
    let source_root_hash = apply_report.source_root_hash.clone();

    if failure_point == Some(SqliteReadCutFailurePoint::DbUnavailable) {
        remove_file_if_exists(db_path)?;
    }
    if failure_point == Some(SqliteReadCutFailurePoint::CorruptDbPathOrSchemaMismatch)
        || failure_point == Some(SqliteReadCutFailurePoint::AfterFallbackSelectedBeforeReportCommit)
    {
        fs::write(db_path, b"not a sqlite database").map_err(|error| {
            format!(
                "write corrupt read-cut db fixture failed {}: {error}",
                db_path.display()
            )
        })?;
    }

    let db_read = read_db_projection(db_path, projection_root)?;
    match db_read {
        DbReadOutcome::Authoritative(export_manifest, projected_files, db_row_counts) => {
            if failure_point
                == Some(SqliteReadCutFailurePoint::AfterDbReadBeforeProjectionVerification)
            {
                return Err(
                    "injected_failure_after_db_read_before_projection_verification".to_string(),
                );
            }
            let mut projection_hash = export_manifest.export_hash.clone();
            if failure_point == Some(SqliteReadCutFailurePoint::ProjectionHashMismatch) {
                projection_hash = "injected_projection_hash_mismatch".to_string();
            }
            if export_manifest.export_hash != projection_hash {
                remove_file_if_exists(read_cut_report_path)?;
                return Err(format!(
                    "read_cut_blocked:projection_hash_mismatch:db={}:projection={}",
                    export_manifest.export_hash, projection_hash
                ));
            }
            write_projection_and_manifest_from_db(
                fixture_root,
                db_path,
                projection_root,
                manifest_path,
                &source_root_hash,
                &export_manifest.projected_files,
                &projected_files,
                &db_row_counts,
                &export_manifest.redaction_manifest,
            )?;
            let manifest = load_completed_rollback_manifest(manifest_path, failure_point)?;
            let rollback_manifest_hash = manifest_hash(&manifest)?;
            let recovery_dry_run = recovery_plan("db_authoritative_success");
            let report = SqliteReadCutRehearsalReport {
                status: "completed".to_string(),
                read_source: "db_authoritative".to_string(),
                fallback_decision: "not_used".to_string(),
                degraded: false,
                source_root_hash,
                db_path_ref: db_path.display().to_string(),
                projection_root_ref: projection_root.display().to_string(),
                manifest_path_ref: manifest_path.display().to_string(),
                read_cut_report_path_ref: read_cut_report_path.display().to_string(),
                db_read_hash: Some(export_manifest.export_hash.clone()),
                projection_hash: Some(projection_hash),
                source_projection_hash: None,
                projected_files,
                db_row_counts,
                rollback_manifest_hash,
                recovery_dry_run,
                failure_point: failure_point.map(|point| format!("{point:?}")),
                redaction_policy: export_manifest.redaction_manifest,
            };
            write_report(read_cut_report_path, &report)?;
            Ok(report)
        }
        DbReadOutcome::Unavailable(reason) => {
            let fallback = verified_json_projection_fallback(
                projection_root,
                manifest_path,
                &source_root_hash,
            )?;
            if failure_point
                == Some(SqliteReadCutFailurePoint::AfterFallbackSelectedBeforeReportCommit)
            {
                remove_file_if_exists(read_cut_report_path)?;
                return Err(
                    "injected_failure_after_fallback_selected_before_report_commit".to_string(),
                );
            }
            let manifest = load_completed_rollback_manifest(manifest_path, failure_point)?;
            let rollback_manifest_hash = manifest_hash(&manifest)?;
            let report = SqliteReadCutRehearsalReport {
                status: "fallback_degraded".to_string(),
                read_source: "json_projection_fallback".to_string(),
                fallback_decision: format!("selected:{reason}"),
                degraded: true,
                source_root_hash,
                db_path_ref: db_path.display().to_string(),
                projection_root_ref: projection_root.display().to_string(),
                manifest_path_ref: manifest_path.display().to_string(),
                read_cut_report_path_ref: read_cut_report_path.display().to_string(),
                db_read_hash: None,
                projection_hash: Some(fallback.projection_hash.clone()),
                source_projection_hash: Some(fallback.projection_hash),
                projected_files: fallback.projected_files,
                db_row_counts: BTreeMap::new(),
                rollback_manifest_hash,
                recovery_dry_run: recovery_plan("json_projection_fallback"),
                failure_point: failure_point.map(|point| format!("{point:?}")),
                redaction_policy: redaction_policy(),
            };
            write_report(read_cut_report_path, &report)?;
            Ok(report)
        }
    }
}

struct LimitedJsonFallback {
    projection_hash: String,
    fallback_root_hash: String,
    manifest_hash: String,
    counts: BTreeMap<String, usize>,
}

struct LimitedDbProjection {
    projection_hash: String,
    counts: BTreeMap<String, usize>,
}

fn validate_limited_read_model(
    read_model_name: &str,
    allowed_read_models: &BTreeSet<String>,
) -> Result<(), String> {
    if read_model_name != WORKFLOW_STATE_SUMMARY_READ_MODEL {
        return Err(format!(
            "limited_read_cut_blocked:unsupported_read_model:{read_model_name}"
        ));
    }
    if !allowed_read_models.contains(read_model_name) {
        return Err(format!(
            "limited_read_cut_blocked:read_model_not_allowed:{read_model_name}"
        ));
    }
    Ok(())
}

fn validate_limited_read_cut_paths(
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    read_cut_report_path: &Path,
    denied_markers: &[String],
) -> Result<(), String> {
    for path in [
        db_path,
        json_fallback_root,
        projection_root,
        rollback_manifest_path,
        read_cut_report_path,
    ] {
        if !path.is_absolute() {
            return Err(format!(
                "limited_read_cut_blocked:absolute_path_required:{}",
                path.display()
            ));
        }
        reject_denied_path_markers(path, denied_markers)?;
    }
    validate_temp_db_path(db_path)?;
    if !json_fallback_root.starts_with(std::env::temp_dir())
        && !json_fallback_root.starts_with(manifest_r3_a10_fixture_root())
    {
        return Err(format!(
            "limited_read_cut_blocked:fallback_root_must_be_temp_or_r3_a10_fixture:{}",
            json_fallback_root.display()
        ));
    }
    validate_limited_projection_paths(
        projection_root,
        rollback_manifest_path,
        read_cut_report_path,
    )?;
    if projection_root.starts_with(json_fallback_root)
        || json_fallback_root.starts_with(projection_root)
    {
        return Err(
            "limited_read_cut_blocked:fallback_and_projection_roots_must_be_separate".to_string(),
        );
    }
    Ok(())
}

fn verified_limited_json_fallback(
    fallback_root: &Path,
    read_model_name: &str,
    expected_fallback_hash: Option<&str>,
    denied_markers: &[String],
    failure_point: Option<SqliteLimitedReadCutFailurePoint>,
) -> Result<LimitedJsonFallback, String> {
    reject_denied_path_markers(fallback_root, denied_markers)?;
    let workflow = load_workflow_state_from_root(fallback_root)?;
    ensure_no_forbidden_limited_value(&workflow)?;
    let summary = workflow_state_summary(&workflow)?;
    let mut projection_hash = canonical_json_hash(&summary);
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::FallbackHashMismatch) {
        projection_hash = "injected_fallback_hash_mismatch".to_string();
    }
    if let Some(expected) = expected_fallback_hash {
        if projection_hash != expected {
            return Err(format!(
                "limited_read_cut_blocked:fallback_hash_mismatch:expected={expected}:actual={projection_hash}"
            ));
        }
    }
    let fallback_root_hash = dir_hash(fallback_root)?;
    let manifest = json!({
        "schema_version": LIMITED_READ_CUT_SCHEMA_VERSION,
        "mode": LIMITED_READ_CUT_MODE,
        "level": LEVEL_A,
        "status": "completed",
        "read_model_name": read_model_name,
        "fallback_root_hash": fallback_root_hash,
        "projection_hash": projection_hash,
        "counts": summary_counts(&summary)?,
        "redaction_policy": redaction_policy()
    });
    Ok(LimitedJsonFallback {
        projection_hash,
        fallback_root_hash,
        manifest_hash: canonical_json_hash(&manifest),
        counts: summary_counts(&summary)?,
    })
}

fn read_limited_db_model(
    db_path: &Path,
    _read_model_name: &str,
    projection_root: &Path,
    failure_point: Option<SqliteLimitedReadCutFailurePoint>,
) -> Result<LimitedDbProjection, String> {
    verify_limited_db_integrity(db_path)
        .map_err(|reason| format!("limited_read_cut_fallback:{reason}"))?;
    let export_manifest =
        export_temp_db_to_json_dry_run(db_path, &projection_root.display().to_string())
            .map_err(|error| format!("limited_read_cut_fallback:db_export_failed:{error}"))?;
    let workflow_projection = export_manifest
        .projected_files
        .iter()
        .find(|file| file.path == "workflow-state.v0.json")
        .ok_or_else(|| "limited_read_cut_blocked:workflow_projection_missing".to_string())?;
    ensure_no_forbidden_limited_value(&workflow_projection.projection)?;
    let summary = workflow_state_summary(&workflow_projection.projection)?;
    let mut projection_hash = canonical_json_hash(&summary);
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::ProjectionHashMismatch) {
        projection_hash = "injected_projection_hash_mismatch".to_string();
    }
    let actual_projection_hash = canonical_json_hash(&summary);
    if projection_hash != actual_projection_hash {
        return Err(format!(
            "limited_read_cut_blocked:projection_hash_mismatch:db={actual_projection_hash}:projection={projection_hash}"
        ));
    }
    Ok(LimitedDbProjection {
        projection_hash,
        counts: summary_counts(&summary)?,
    })
}

fn verify_limited_db_integrity(db_path: &Path) -> Result<(), String> {
    match verify_db_integrity(db_path) {
        Ok(()) => Ok(()),
        Err(reason) if reason.contains("missing_db_path") => {
            Err(format!("db_unavailable:{reason}"))
        }
        Err(reason) if reason.contains("schema_mismatch") => {
            Err(format!("db_schema_mismatch:{reason}"))
        }
        Err(reason) => Err(format!("db_integrity_failure:{reason}")),
    }
}

fn validate_limited_projection_paths(
    projection_root: &Path,
    manifest_path: &Path,
    read_cut_report_path: &Path,
) -> Result<(), String> {
    if !projection_root.is_absolute()
        || !manifest_path.is_absolute()
        || !read_cut_report_path.is_absolute()
    {
        return Err(
            "limited_read_cut_blocked: projection/report paths must be absolute".to_string(),
        );
    }
    let fixture_root = manifest_r3_a10_fixture_root();
    let projection_allowed = projection_root.starts_with(std::env::temp_dir())
        || projection_root.starts_with(fixture_root);
    if !projection_allowed
        || !manifest_path.starts_with(projection_root)
        || !read_cut_report_path.starts_with(projection_root)
    {
        return Err(format!(
            "limited_read_cut_blocked: refusing projection/manifest/report outside temp or R3-A10 fixtures: {} / {} / {}",
            projection_root.display(),
            manifest_path.display(),
            read_cut_report_path.display()
        ));
    }
    Ok(())
}

fn limited_read_cut_report(
    status: &str,
    read_model_name: &str,
    feature_flag_enabled: bool,
    read_source: &str,
    fallback_decision: &str,
    degraded: bool,
    db_path: &Path,
    fallback_root_hash: &str,
    projection_hash: &str,
    rollback_manifest_hash: &str,
    expected_db_hash: Option<&str>,
    actual_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    actual_fallback_hash: &str,
    counts: BTreeMap<String, usize>,
    safety_flags: SqliteLimitedReadCutSafetyFlags,
    failure_point: Option<SqliteLimitedReadCutFailurePoint>,
) -> SqliteLimitedReadCutReport {
    SqliteLimitedReadCutReport {
        schema_version: LIMITED_READ_CUT_SCHEMA_VERSION.to_string(),
        mode: LIMITED_READ_CUT_MODE.to_string(),
        level: LEVEL_A.to_string(),
        status: status.to_string(),
        read_model_name: read_model_name.to_string(),
        feature_flag_enabled,
        read_source: read_source.to_string(),
        fallback_decision: fallback_decision.to_string(),
        degraded,
        db_path_hash: path_hash(db_path),
        fallback_root_hash: fallback_root_hash.to_string(),
        projection_hash: projection_hash.to_string(),
        rollback_manifest_hash: rollback_manifest_hash.to_string(),
        expected_db_hash: expected_db_hash.map(ToString::to_string),
        actual_db_hash: actual_db_hash.map(ToString::to_string),
        expected_fallback_hash: expected_fallback_hash.map(ToString::to_string),
        actual_fallback_hash: actual_fallback_hash.to_string(),
        counts,
        recovery_dry_run: limited_recovery_plan(read_source),
        safety_flags,
        failure_point: failure_point.map(|point| format!("{point:?}")),
        redaction_policy: redaction_policy(),
    }
}

fn write_limited_read_cut_report(
    path: &Path,
    report: &SqliteLimitedReadCutReport,
) -> Result<(), String> {
    if report.status != "completed"
        && report.status != "fallback_degraded"
        && report.status != "feature_flag_disabled_fallback"
    {
        return Err(format!(
            "limited_read_cut_report_status_not_committable:{}",
            report.status
        ));
    }
    if report.safety_flags.product_global_read_path_changed
        || report.safety_flags.app_startup_reads_db
        || report.safety_flags.tauri_command_reads_db
        || report.safety_flags.ui_reads_db
        || report.safety_flags.stop_write_json
        || report.safety_flags.source_json_written
        || report.safety_flags.production_restore_performed
        || report.safety_flags.codex_home_touched
    {
        return Err("limited_read_cut_forbidden_safety_flag_true".to_string());
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize limited read-cut report failed: {error}"))?,
    )
}

fn write_limited_rollback_manifest(
    path: &Path,
    projection: &LimitedDbProjection,
) -> Result<String, String> {
    let manifest = json!({
        "schema_version": LIMITED_READ_CUT_SCHEMA_VERSION,
        "mode": LIMITED_READ_CUT_MODE,
        "level": LEVEL_A,
        "status": "completed",
        "projection_hash": projection.projection_hash,
        "counts": projection.counts,
        "recovery_dry_run": limited_recovery_plan("rollback_manifest")
    });
    let manifest_hash = canonical_json_hash(&manifest);
    let mut value = manifest;
    value["rollback_manifest_hash"] = Value::String(manifest_hash.clone());
    write_json_file(path, &value)?;
    Ok(manifest_hash)
}

fn verify_limited_rollback_manifest(
    path: &Path,
    failure_point: Option<SqliteLimitedReadCutFailurePoint>,
    prefix: &str,
) -> Result<String, String> {
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::MissingRollbackManifest) {
        remove_file_if_exists(path)?;
    }
    if failure_point == Some(SqliteLimitedReadCutFailurePoint::IncompleteRollbackManifest) {
        write_json_file(
            path,
            &json!({
                "schema_version": LIMITED_READ_CUT_SCHEMA_VERSION,
                "status": "manifest_commit_failed_before_complete",
                "production_restore_performed": false
            }),
        )?;
    }
    let manifest = load_manifest_file(path).map_err(|error| format!("{prefix}:{error}"))?;
    if manifest.get("status").and_then(Value::as_str) != Some("completed") {
        remove_file_if_exists(path)?;
        return Err(format!("{prefix}:rollback_manifest_incomplete"));
    }
    manifest
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("{prefix}:rollback_manifest_hash_missing"))
}

fn limited_recovery_plan(_reason: &str) -> SqliteLimitedReadCutRecoveryDryRun {
    SqliteLimitedReadCutRecoveryDryRun {
        status: "recovery_dry_run_only".to_string(),
        would_disable_limited_read_cut: true,
        would_use_json_fallback: true,
        would_preserve_db_for_audit: true,
        would_require_supervisor_decision: true,
        production_restore_performed: false,
        instructions: vec![
            "would disable limited read-cut before any recovery action".to_string(),
            "would use verified JSON fallback when DB read is degraded or feature flag is off"
                .to_string(),
            "would preserve DB for audit; no production restore is performed here".to_string(),
            "would require supervisor decision before Level B or production path changes"
                .to_string(),
        ],
    }
}

fn load_workflow_state_from_root(root: &Path) -> Result<Value, String> {
    let path = root.join("workflow-state.v0.json");
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "limited_read_cut_blocked:fallback_workflow_state_missing:{}:{error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "limited_read_cut_blocked:fallback_workflow_state_corrupt:{}:{error}",
            path.display()
        )
    })
}

fn workflow_state_summary(value: &Value) -> Result<Value, String> {
    let projects = array_count(value, "projects");
    let workflows = array_count(value, "workflows");
    let work_items = array_count(value, "work_items");
    let audit_events = array_count(value, "audit_events");
    let revision = value.get("revision").and_then(Value::as_i64).unwrap_or(1);
    Ok(json!({
        "schema_version": "workflow_state_summary.v1",
        "source_schema_version": value.get("schema_version").and_then(Value::as_str).unwrap_or("unknown"),
        "revision": revision,
        "projects": projects,
        "workflows": workflows,
        "work_items": work_items,
        "audit_events": audit_events
    }))
}

fn summary_counts(summary: &Value) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for key in ["projects", "workflows", "work_items", "audit_events"] {
        let value = summary
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("limited_read_cut_blocked:summary_count_missing:{key}"))?;
        counts.insert(key.to_string(), value as usize);
    }
    Ok(counts)
}

fn array_count(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn ensure_no_forbidden_limited_value(value: &Value) -> Result<(), String> {
    let text = value.to_string().to_ascii_lowercase();
    for marker in [
        "provider credential value",
        "full transcript body",
        "rollout body payload",
        "\"prompt_body\"",
        "\"token\"",
        "\"secret\"",
    ] {
        if text.contains(marker) {
            return Err(format!(
                "limited_read_cut_blocked:forbidden_projection_body:{marker}"
            ));
        }
    }
    Ok(())
}

fn effective_limited_denied_markers(markers: &[String]) -> Vec<String> {
    let mut values = DEFAULT_LIMITED_READ_CUT_DENIED_PATH_MARKERS
        .iter()
        .map(|marker| (*marker).to_string())
        .collect::<Vec<_>>();
    values.extend(markers.iter().cloned());
    values.sort();
    values.dedup();
    values
}

fn reject_denied_path_markers(path: &Path, denied_markers: &[String]) -> Result<(), String> {
    let normalized = path.display().to_string().to_ascii_lowercase();
    for marker in denied_markers {
        if normalized.contains(&marker.to_ascii_lowercase()) {
            return Err(format!(
                "limited_read_cut_blocked:denied_path_marker:{}:{}",
                marker,
                path.display()
            ));
        }
    }
    Ok(())
}

fn file_hash(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "limited_read_cut_blocked:db_hash_read_failed:{}:{error}",
            path.display()
        )
    })?;
    Ok(sha256_hex(&bytes))
}

fn dir_hash(root: &Path) -> Result<String, String> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("read dir for hash failed {}: {error}", root.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read dir entry for hash failed {}: {error}", root.display()))?;
    entries.sort();
    let mut manifest = Vec::new();
    for path in entries.into_iter().filter(|path| path.is_file()) {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        manifest.push(json!({
            "path": name,
            "hash": file_hash(&path)?
        }));
    }
    Ok(canonical_json_hash(&json!(manifest)))
}

fn path_hash(path: &Path) -> String {
    canonical_json_hash(&json!({ "path_ref": path.display().to_string() }))
}

fn manifest_r3_a10_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a10")
}

enum DbReadOutcome {
    Authoritative(
        crate::workbench_sqlite_exporter::SqliteExportDryRunManifest,
        Vec<SqliteReadCutProjectedFile>,
        BTreeMap<String, i64>,
    ),
    Unavailable(String),
}

struct JsonProjectionFallback {
    projection_hash: String,
    projected_files: Vec<SqliteReadCutProjectedFile>,
}

fn read_db_projection(db_path: &Path, projection_root: &Path) -> Result<DbReadOutcome, String> {
    match verify_db_integrity(db_path) {
        Ok(()) => {}
        Err(reason) => return Ok(DbReadOutcome::Unavailable(reason)),
    }
    let export_manifest =
        match export_temp_db_to_json_dry_run(db_path, &projection_root.display().to_string()) {
            Ok(manifest) => manifest,
            Err(error) => {
                return Ok(DbReadOutcome::Unavailable(format!(
                    "db_export_failed:{error}"
                )))
            }
        };
    let projected_files = projected_files_from_export(&export_manifest.projected_files);
    let db_row_counts = db_row_counts(db_path)?;
    Ok(DbReadOutcome::Authoritative(
        export_manifest,
        projected_files,
        db_row_counts,
    ))
}

fn verify_db_integrity(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Err("db_unavailable:missing_db_path".to_string());
    }
    let connection =
        Connection::open(db_path).map_err(|error| format!("db_unavailable:{error}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("db_integrity_check_failed:{error}"))?;
    if integrity != "ok" {
        return Err(format!("db_integrity_check_failed:{integrity}"));
    }
    let schema_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("db_schema_mismatch:{error}"))?;
    if schema_count != 1 {
        return Err("db_schema_mismatch:missing_schema_migrations".to_string());
    }
    Ok(())
}

fn verified_json_projection_fallback(
    projection_root: &Path,
    manifest_path: &Path,
    source_root_hash: &str,
) -> Result<JsonProjectionFallback, String> {
    let manifest = load_manifest_file(manifest_path)?;
    if manifest.get("status").and_then(Value::as_str) != Some("completed") {
        return Err("read_cut_blocked:fallback_projection_manifest_incomplete".to_string());
    }
    if manifest.get("source_root_hash").and_then(Value::as_str) != Some(source_root_hash) {
        return Err("read_cut_blocked:fallback_source_hash_mismatch".to_string());
    }
    let projected_files = manifest
        .get("projected_files")
        .and_then(Value::as_array)
        .ok_or_else(|| "read_cut_blocked:fallback_projected_files_missing".to_string())?
        .iter()
        .map(projected_file_from_manifest_entry)
        .collect::<Result<Vec<_>, _>>()?;
    for file in &projected_files {
        verify_projected_file_hash(projection_root, file)?;
    }
    let projection_input = json!({
        "source_root_hash": source_root_hash,
        "projected_files": projected_files,
        "fallback_kind": "json_projection_verified"
    });
    Ok(JsonProjectionFallback {
        projection_hash: canonical_json_hash(&projection_input),
        projected_files,
    })
}

fn load_completed_rollback_manifest(
    manifest_path: &Path,
    failure_point: Option<SqliteReadCutFailurePoint>,
) -> Result<Value, String> {
    if failure_point == Some(SqliteReadCutFailurePoint::MissingRollbackManifest) {
        remove_file_if_exists(manifest_path)?;
    }
    if failure_point == Some(SqliteReadCutFailurePoint::IncompleteRollbackManifest) {
        write_json_file(
            manifest_path,
            &json!({
                "schema_version": "workbench_sqlite_dual_write_rehearsal.v1",
                "status": "manifest_commit_failed_before_complete",
                "failure_point": "IncompleteRollbackManifest"
            }),
        )?;
    }
    let manifest = load_manifest_file(manifest_path)?;
    if manifest.get("status").and_then(Value::as_str) != Some("completed") {
        remove_file_if_exists(manifest_path)?;
        return Err("read_cut_blocked:rollback_manifest_incomplete".to_string());
    }
    if manifest
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err("read_cut_blocked:rollback_manifest_hash_missing".to_string());
    }
    Ok(manifest)
}

fn load_manifest_file(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("read_cut_blocked:rollback_manifest_missing:{error}"))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("read_cut_blocked:rollback_manifest_corrupt:{error}"))
}

fn manifest_hash(manifest: &Value) -> Result<String, String> {
    manifest
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| "read_cut_blocked:rollback_manifest_hash_missing".to_string())
}

fn projected_files_from_export(files: &[SqliteProjectedFile]) -> Vec<SqliteReadCutProjectedFile> {
    files
        .iter()
        .map(|file| SqliteReadCutProjectedFile {
            path: file.path.clone(),
            projected_hash: file.projected_hash.clone(),
            record_count: file.record_count,
        })
        .collect()
}

fn projected_file_from_manifest_entry(value: &Value) -> Result<SqliteReadCutProjectedFile, String> {
    Ok(SqliteReadCutProjectedFile {
        path: value
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| "read_cut_blocked:fallback_projected_file_path_missing".to_string())?
            .to_string(),
        projected_hash: value
            .get("projected_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| "read_cut_blocked:fallback_projected_file_hash_missing".to_string())?
            .to_string(),
        record_count: value
            .get("record_count")
            .and_then(Value::as_u64)
            .ok_or_else(|| "read_cut_blocked:fallback_projected_file_count_missing".to_string())?
            as usize,
    })
}

fn verify_projected_file_hash(
    projection_root: &Path,
    file: &SqliteReadCutProjectedFile,
) -> Result<(), String> {
    if file.path.contains('/') || file.path.contains('\\') {
        return Err(format!(
            "read_cut_blocked:fallback_projection_path_not_flat:{}",
            file.path
        ));
    }
    let bytes = fs::read(projection_root.join(&file.path)).map_err(|error| {
        format!(
            "read_cut_blocked:fallback_projection_file_missing:{}:{error}",
            file.path
        )
    })?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "read_cut_blocked:fallback_projection_file_corrupt:{}:{error}",
            file.path
        )
    })?;
    let actual = canonical_json_hash(&value);
    if actual != file.projected_hash {
        return Err(format!(
            "read_cut_blocked:fallback_projection_hash_mismatch:{}",
            file.path
        ));
    }
    Ok(())
}

fn write_projection_and_manifest_from_db(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    manifest_path: &Path,
    source_root_hash: &str,
    export_files: &[SqliteProjectedFile],
    projected_files: &[SqliteReadCutProjectedFile],
    db_row_counts: &BTreeMap<String, i64>,
    redaction_manifest: &[String],
) -> Result<(), String> {
    fs::create_dir_all(projection_root).map_err(|error| {
        format!(
            "create sqlite read-cut projection root failed {}: {error}",
            projection_root.display()
        )
    })?;
    remove_file_if_exists(manifest_path)?;
    for file in export_files {
        write_projected_file(projection_root, file)?;
    }
    let rollback_manifest = rollback_manifest_value(
        fixture_root,
        db_path,
        projection_root,
        manifest_path,
        source_root_hash,
        projected_files,
        db_row_counts,
        redaction_manifest,
    );
    let temp_manifest = temp_manifest_path(manifest_path);
    write_json_file(&temp_manifest, &rollback_manifest)?;
    fs::rename(&temp_manifest, manifest_path).map_err(|error| {
        format!(
            "commit sqlite read-cut rollback manifest failed {} -> {}: {error}",
            temp_manifest.display(),
            manifest_path.display()
        )
    })
}

fn write_projected_file(root: &Path, file: &SqliteProjectedFile) -> Result<(), String> {
    if file.path.contains('/') || file.path.contains('\\') {
        return Err(format!("projection_file_name_must_be_flat: {}", file.path));
    }
    write_json_file(&root.join(&file.path), &file.projection)
}

fn rollback_manifest_value(
    fixture_root: &Path,
    db_path: &Path,
    projection_root: &Path,
    manifest_path: &Path,
    source_root_hash: &str,
    projected_files: &[SqliteReadCutProjectedFile],
    db_row_counts: &BTreeMap<String, i64>,
    redaction_manifest: &[String],
) -> Value {
    let payload = json!({
        "schema_version": "workbench_sqlite_read_cut_rehearsal.v1",
        "status": "completed",
        "source_root_ref": fixture_root.display().to_string(),
        "source_root_hash": source_root_hash,
        "db_path_ref": db_path.display().to_string(),
        "projection_root_ref": projection_root.display().to_string(),
        "manifest_path_ref": manifest_path.display().to_string(),
        "projected_files": projected_files,
        "db_row_counts": db_row_counts,
        "redaction_policy": redaction_manifest,
        "recovery_dry_run": recovery_plan("rollback_manifest")
    });
    let payload_hash = canonical_json_hash(&payload);
    let mut manifest = payload;
    manifest["rollback_manifest_hash"] = Value::String(payload_hash);
    manifest
}

fn temp_manifest_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
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

fn recovery_plan(reason: &str) -> SqliteReadCutRecoveryDryRun {
    SqliteReadCutRecoveryDryRun {
        status: "recovery_dry_run_only".to_string(),
        would_use_db: reason == "db_authoritative_success",
        would_use_json_projection: reason != "db_authoritative_success",
        would_disable_db_read_cut: reason != "db_authoritative_success",
        would_preserve_db_for_audit: true,
        production_restore_performed: false,
        instructions: vec![
            "would verify rollback manifest before any restore decision".to_string(),
            "would use JSON projection fallback only when DB read is degraded".to_string(),
            "would disable DB read-cut before recovery in production planning".to_string(),
            "would preserve DB for audit; no production restore is performed here".to_string(),
        ],
    }
}

fn validate_fixture_root(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!("r3_a4_fixture_dir_required: {}", path.display()));
    }
    let fixture_root = manifest_r3_a4_fixture_root();
    if path.starts_with(&fixture_root) || path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_or_r3_a4_fixture_path_required: refusing fixture source outside temp or R3-A4 fixtures: {}",
            path.display()
        ))
    }
}

fn validate_temp_db_path(path: &Path) -> Result<(), String> {
    if path.is_absolute() && path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "temp_db_path_required: refusing read-cut rehearsal db outside temp: {}",
            path.display()
        ))
    }
}

fn validate_projection_paths(
    projection_root: &Path,
    manifest_path: &Path,
    read_cut_report_path: &Path,
) -> Result<(), String> {
    if !projection_root.is_absolute()
        || !manifest_path.is_absolute()
        || !read_cut_report_path.is_absolute()
    {
        return Err(
            "temp_or_r3_a4_fixture_path_required: projection/report paths must be absolute".into(),
        );
    }
    let fixture_root = manifest_r3_a4_fixture_root();
    let projection_allowed = projection_root.starts_with(std::env::temp_dir())
        || projection_root.starts_with(fixture_root);
    if !projection_allowed
        || !manifest_path.starts_with(projection_root)
        || !read_cut_report_path.starts_with(projection_root)
    {
        return Err(format!(
            "temp_or_r3_a4_fixture_path_required: refusing projection/manifest/report outside temp or R3-A4 fixtures: {} / {} / {}",
            projection_root.display(),
            manifest_path.display(),
            read_cut_report_path.display()
        ));
    }
    Ok(())
}

fn write_report(path: &Path, report: &SqliteReadCutRehearsalReport) -> Result<(), String> {
    if report.status != "completed" && report.status != "fallback_degraded" {
        return Err(format!(
            "read_cut_report_status_not_committable:{}",
            report.status
        ));
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize read-cut report value failed: {error}"))?,
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

fn redaction_policy() -> Vec<String> {
    vec![
        "prompt_body:omitted".to_string(),
        "full_transcript:omitted".to_string(),
        "secret_token_credential_keychain_oauth_provider_credential:omitted".to_string(),
        "rollout_body:omitted".to_string(),
    ]
}

fn manifest_r3_a4_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a4")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_read_cut_db_authoritative_success_uses_db_projection_hash() {
        let fixture = fixture_dir("read-cut-valid-core-chain");
        let paths = prepare_paths("db-success");

        let report = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            None,
        )
        .expect("read-cut success");

        assert_eq!(report.status, "completed");
        assert_eq!(report.read_source, "db_authoritative");
        assert_eq!(report.fallback_decision, "not_used");
        assert_eq!(report.db_read_hash, report.projection_hash);
        assert!(paths.read_cut_report_path.exists());
        let report_text = fs::read_to_string(&paths.read_cut_report_path).expect("read report");
        assert!(report_text.contains("\"read_source\":\"db_authoritative\""));
        assert!(!report_text.contains("json_projection_fallback"));
    }

    #[test]
    fn sqlite_read_cut_idempotent_rerun_keeps_stable_report() {
        let fixture = fixture_dir("read-cut-idempotent-rerun");
        let paths = prepare_paths("idempotent");

        rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            None,
        )
        .expect("first read-cut");
        let first_report = fs::read_to_string(&paths.read_cut_report_path).expect("first report");
        rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            None,
        )
        .expect("second read-cut");
        let second_report = fs::read_to_string(&paths.read_cut_report_path).expect("second report");

        assert_eq!(first_report, second_report);
    }

    #[test]
    fn sqlite_read_cut_db_unavailable_uses_verified_json_projection_fallback() {
        let fixture = fixture_dir("read-cut-db-unavailable-json-fallback");
        let paths = prepare_paths("db-unavailable");
        prepare_projection_manifest(&fixture, &paths);
        fs::remove_file(&paths.db_path).expect("remove db for unavailable fallback");

        let report = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::DbUnavailable),
        )
        .expect("fallback read-cut");

        assert_eq!(report.status, "fallback_degraded");
        assert_eq!(report.read_source, "json_projection_fallback");
        assert!(report.fallback_decision.contains("selected:"));
        assert!(report.degraded);
        assert!(report.db_read_hash.is_none());
        assert!(paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_read_cut_schema_mismatch_fallback_is_degraded_not_db_success() {
        let fixture = fixture_dir("read-cut-db-schema-mismatch-fallback");
        let paths = prepare_paths("schema-mismatch");
        prepare_projection_manifest(&fixture, &paths);

        let report = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::CorruptDbPathOrSchemaMismatch),
        )
        .expect("schema mismatch fallback");

        assert_eq!(report.status, "fallback_degraded");
        assert_eq!(report.read_source, "json_projection_fallback");
        assert_ne!(report.fallback_decision, "not_used");
        let report_text = fs::read_to_string(&paths.read_cut_report_path).expect("read report");
        assert!(!report_text.contains("\"status\":\"completed\""));
        assert!(!report_text.contains("\"read_source\":\"db_authoritative\""));
    }

    #[test]
    fn sqlite_read_cut_projection_hash_mismatch_blocks_without_completed_report() {
        let fixture = fixture_dir("read-cut-projection-hash-mismatch-blocked");
        let paths = prepare_paths("hash-mismatch");

        let err = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::ProjectionHashMismatch),
        )
        .expect_err("hash mismatch should block");

        assert!(err.contains("read_cut_blocked:projection_hash_mismatch"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_read_cut_missing_manifest_blocks_without_completed_report() {
        let fixture = fixture_dir("read-cut-missing-manifest-blocked");
        let paths = prepare_paths("missing-manifest");

        let err = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::MissingRollbackManifest),
        )
        .expect_err("missing manifest should block");

        assert!(err.contains("rollback_manifest_missing"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_read_cut_incomplete_manifest_blocks_without_completed_report() {
        let fixture = fixture_dir("read-cut-incomplete-manifest-blocked");
        let paths = prepare_paths("incomplete-manifest");

        let err = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::IncompleteRollbackManifest),
        )
        .expect_err("incomplete manifest should block");

        assert!(err.contains("rollback_manifest_incomplete"));
        assert!(!paths.read_cut_report_path.exists());
        assert!(!paths.manifest_path.exists());
    }

    #[test]
    fn sqlite_read_cut_failure_injection_before_db_read_creates_no_report() {
        let fixture = fixture_dir("read-cut-valid-core-chain");
        let paths = prepare_paths("before-db-read");

        let err = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::BeforeDbRead),
        )
        .expect_err("before db read failure");

        assert!(err.contains("injected_failure_before_db_read"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_read_cut_failure_after_db_read_before_verification_creates_no_report() {
        let fixture = fixture_dir("read-cut-valid-core-chain");
        let paths = prepare_paths("after-db-read");

        let err = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::AfterDbReadBeforeProjectionVerification),
        )
        .expect_err("after db read before projection verification failure");

        assert!(err.contains("after_db_read_before_projection_verification"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_read_cut_failure_after_fallback_before_report_commit_creates_no_report() {
        let fixture = fixture_dir("read-cut-db-unavailable-json-fallback");
        let paths = prepare_paths("fallback-before-report");
        prepare_projection_manifest(&fixture, &paths);

        let err = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            Some(SqliteReadCutFailurePoint::AfterFallbackSelectedBeforeReportCommit),
        )
        .expect_err("fallback before report commit failure");

        assert!(err.contains("after_fallback_selected_before_report_commit"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_read_cut_report_and_projection_omit_forbidden_sensitive_fields() {
        let fixture = fixture_dir("read-cut-sensitive-redaction");
        let paths = prepare_paths("sensitive");

        rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            None,
        )
        .expect("read-cut sensitive redaction");

        let mut text = fs::read_to_string(&paths.read_cut_report_path).expect("read report");
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
    fn sqlite_read_cut_recovery_dry_run_does_not_restore_outputs() {
        let fixture = fixture_dir("rollback-read-cut-recovery-dry-run");
        let paths = prepare_paths("recovery");

        let report = rehearse_fixture_read_cut(
            &fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &paths.read_cut_report_path,
            None,
        )
        .expect("recovery dry-run");

        assert_eq!(report.recovery_dry_run.status, "recovery_dry_run_only");
        assert!(!report.recovery_dry_run.production_restore_performed);
        assert!(report.recovery_dry_run.would_preserve_db_for_audit);
        let restored_outputs = fs::read_dir(&paths.projection_root)
            .expect("read projection dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("restored-"))
            .count();
        assert_eq!(restored_outputs, 0);
    }

    #[test]
    fn sqlite_limited_read_cut_feature_flag_disabled_uses_fallback_without_db() {
        let paths = prepare_limited_paths("flag-disabled");
        prepare_limited_fallback(&paths);
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let report = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            false,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            None,
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect("feature flag disabled fallback");

        assert_eq!(report.status, "feature_flag_disabled_fallback");
        assert_eq!(report.read_source, "json_fallback");
        assert!(!report.safety_flags.limited_read_cut_enabled);
        assert!(!paths.db_path.exists());
        assert!(paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_db_authoritative_success() {
        let paths = prepare_limited_paths("db-success");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let report = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect("db limited success");

        assert_eq!(report.status, "completed");
        assert_eq!(report.read_source, "db_limited");
        assert!(report.safety_flags.limited_read_cut_enabled);
        assert!(!report.safety_flags.product_global_read_path_changed);
        assert!(!report.safety_flags.app_startup_reads_db);
        assert!(!report.safety_flags.tauri_command_reads_db);
        assert!(!report.safety_flags.ui_reads_db);
        assert!(!report.safety_flags.stop_write_json);
        assert!(report.recovery_dry_run.would_disable_limited_read_cut);
        assert!(report.recovery_dry_run.would_use_json_fallback);
        assert!(report.recovery_dry_run.would_preserve_db_for_audit);
        assert!(report.recovery_dry_run.would_require_supervisor_decision);
        assert!(!report.recovery_dry_run.production_restore_performed);
        assert_eq!(report.counts.get("projects"), Some(&1));
    }

    #[test]
    fn sqlite_limited_read_cut_rejects_r3_a4_projection_root() {
        let paths = prepare_limited_paths("reject-r3-a4-projection");
        prepare_limited_fallback(&paths);
        let r3_a4_projection_root =
            manifest_r3_a4_fixture_root().join("limited-read-cut-not-allowed");

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            false,
            &paths.db_path,
            &paths.fallback_root,
            &r3_a4_projection_root,
            &r3_a4_projection_root.join("limited-rollback-manifest.json"),
            &r3_a4_projection_root.join("limited-read-cut-report.json"),
            None,
            Some(&limited_expected_fallback_hash(&paths.fallback_root)),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect_err("R3-A4 projection root should be rejected for A10");

        assert!(err.contains("R3-A10 fixtures"));
    }

    #[test]
    fn sqlite_limited_read_cut_db_unavailable_fallback_is_degraded() {
        let paths = prepare_limited_paths("db-unavailable");
        prepare_limited_fallback(&paths);
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let report = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            None,
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::DbUnavailable),
        )
        .expect("db unavailable fallback");

        assert_eq!(report.status, "fallback_degraded");
        assert_eq!(report.read_source, "json_fallback");
        assert!(report.fallback_decision.contains("db_unavailable"));
        assert!(report.degraded);
        assert!(!report.safety_flags.limited_read_cut_enabled);
    }

    #[test]
    fn sqlite_limited_read_cut_schema_mismatch_fallback_is_degraded() {
        let paths = prepare_limited_paths("schema-mismatch");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let report = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            None,
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::DbSchemaMismatch),
        )
        .expect("schema mismatch fallback");

        assert_eq!(report.status, "fallback_degraded");
        assert!(report.fallback_decision.contains("db_schema_mismatch"));
        assert!(!report.safety_flags.limited_read_cut_enabled);
    }

    #[test]
    fn sqlite_limited_read_cut_integrity_failure_fallback_is_degraded() {
        let paths = prepare_limited_paths("integrity-failure");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let report = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            None,
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::DbIntegrityFailure),
        )
        .expect("integrity failure fallback");

        assert_eq!(report.status, "fallback_degraded");
        assert!(report.fallback_decision.contains("db_integrity_failure"));
        assert!(!report.safety_flags.limited_read_cut_enabled);
    }

    #[test]
    fn sqlite_limited_read_cut_db_hash_mismatch_blocks_without_report() {
        let paths = prepare_limited_paths("db-hash-mismatch");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some("sha256:not-the-db"),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect_err("db hash mismatch blocks");

        assert!(err.contains("limited_read_cut_blocked:db_hash_mismatch"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_fallback_hash_mismatch_blocks_without_report() {
        let paths = prepare_limited_paths("fallback-hash-mismatch");
        prepare_limited_fallback(&paths);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            false,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            None,
            Some("sha256:not-the-fallback"),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect_err("fallback hash mismatch blocks");

        assert!(err.contains("limited_read_cut_blocked:fallback_hash_mismatch"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_projection_hash_mismatch_blocks_without_report() {
        let paths = prepare_limited_paths("projection-hash-mismatch");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::ProjectionHashMismatch),
        )
        .expect_err("projection hash mismatch blocks");

        assert!(err.contains("limited_read_cut_blocked:projection_hash_mismatch"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_missing_manifest_blocks_without_report() {
        let paths = prepare_limited_paths("missing-manifest");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::MissingRollbackManifest),
        )
        .expect_err("missing manifest blocks");

        assert!(err.contains("rollback_manifest_missing"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_incomplete_manifest_blocks_without_report() {
        let paths = prepare_limited_paths("incomplete-manifest");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::IncompleteRollbackManifest),
        )
        .expect_err("incomplete manifest blocks");

        assert!(err.contains("rollback_manifest_incomplete"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_failures_before_report_commit_leave_no_completed_report() {
        let paths = prepare_limited_paths("before-report");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::AfterDbReadBeforeReportCommit),
        )
        .expect_err("after db read before report commit fails");

        assert!(err.contains("after_db_read_before_report_commit"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_after_fallback_before_report_commit_leaves_no_report() {
        let paths = prepare_limited_paths("fallback-before-report");
        prepare_limited_fallback(&paths);
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            None,
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            Some(SqliteLimitedReadCutFailurePoint::AfterFallbackSelectedBeforeReportCommit),
        )
        .expect_err("after fallback before report commit fails");

        assert!(err.contains("after_fallback_selected_before_report_commit"));
        assert!(!paths.read_cut_report_path.exists());
    }

    #[test]
    fn sqlite_limited_read_cut_sensitive_redaction_and_idempotent_report() {
        let paths = prepare_limited_paths("sensitive-idempotent");
        prepare_limited_fallback(&paths);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
            .expect("prepare limited db");
        let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
        let expected_fallback_hash = limited_expected_fallback_hash(&paths.fallback_root);

        rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect("first limited read-cut");
        let first = fs::read_to_string(&paths.read_cut_report_path).expect("first report");
        rehearse_limited_read_cut_level_a(
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            true,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.rollback_manifest_path,
            &paths.read_cut_report_path,
            Some(&expected_db_hash),
            Some(&expected_fallback_hash),
            &limited_allowed_models(),
            &[],
            None,
        )
        .expect("second limited read-cut");
        let second = fs::read_to_string(&paths.read_cut_report_path).expect("second report");

        assert_eq!(first, second);
        for forbidden in [
            "provider credential value",
            "full transcript body",
            "rollout body payload",
            "\"prompt_body\"",
        ] {
            assert!(!second.contains(forbidden));
        }
        assert!(second.contains("prompt_body:omitted"));
    }

    struct RehearsalPaths {
        db_path: PathBuf,
        projection_root: PathBuf,
        manifest_path: PathBuf,
        read_cut_report_path: PathBuf,
    }

    fn prepare_paths(name: &str) -> RehearsalPaths {
        let projection_root = temp_projection_root(name);
        RehearsalPaths {
            db_path: temp_db(name),
            manifest_path: projection_root.join("rollback-manifest.json"),
            read_cut_report_path: projection_root.join("read-cut-report.json"),
            projection_root,
        }
    }

    fn prepare_projection_manifest(fixture: &Path, paths: &RehearsalPaths) {
        let apply_report = apply_fixture_dir_to_temp_db(fixture, &paths.db_path, None)
            .expect("prepare fallback DB");
        let export_manifest = export_temp_db_to_json_dry_run(
            &paths.db_path,
            &paths.projection_root.display().to_string(),
        )
        .expect("prepare fallback export");
        let projected_files = projected_files_from_export(&export_manifest.projected_files);
        let db_row_counts = db_row_counts(&paths.db_path).expect("fallback db row counts");
        write_projection_and_manifest_from_db(
            fixture,
            &paths.db_path,
            &paths.projection_root,
            &paths.manifest_path,
            &apply_report.source_root_hash,
            &export_manifest.projected_files,
            &projected_files,
            &db_row_counts,
            &export_manifest.redaction_manifest,
        )
        .expect("prepare projection manifest");
    }

    fn fixture_dir(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("r3-a4")
            .join(name)
    }

    fn temp_db(name: &str) -> PathBuf {
        let nanos = unique_nanos();
        std::env::temp_dir().join(format!("r3-a4-{name}-{nanos}.sqlite"))
    }

    fn temp_projection_root(name: &str) -> PathBuf {
        let nanos = unique_nanos();
        std::env::temp_dir().join(format!("r3-a4-{name}-{nanos}"))
    }

    fn unique_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    }

    struct LimitedPaths {
        db_path: PathBuf,
        fallback_root: PathBuf,
        projection_root: PathBuf,
        rollback_manifest_path: PathBuf,
        read_cut_report_path: PathBuf,
    }

    fn prepare_limited_paths(name: &str) -> LimitedPaths {
        let nanos = unique_nanos();
        let root = std::env::temp_dir().join(format!("r3-a10-{name}-{nanos}"));
        let fallback_root = root.join("fallback");
        let projection_root = root.join("projection");
        LimitedPaths {
            db_path: root.join("limited-read-cut.sqlite"),
            rollback_manifest_path: projection_root.join("limited-rollback-manifest.json"),
            read_cut_report_path: projection_root.join("limited-read-cut-report.json"),
            projection_root,
            fallback_root,
        }
    }

    fn prepare_limited_fallback(paths: &LimitedPaths) {
        fs::create_dir_all(&paths.fallback_root).expect("create fallback root");
        fs::copy(
            fixture_dir_a10("limited-read-cut-workflow-summary").join("workflow-state.v0.json"),
            paths.fallback_root.join("workflow-state.v0.json"),
        )
        .expect("copy fallback workflow state");
    }

    fn limited_expected_fallback_hash(fallback_root: &Path) -> String {
        let workflow = load_workflow_state_from_root(fallback_root).expect("fallback workflow");
        let summary = workflow_state_summary(&workflow).expect("workflow summary");
        canonical_json_hash(&summary)
    }

    fn limited_allowed_models() -> BTreeSet<String> {
        BTreeSet::from([WORKFLOW_STATE_SUMMARY_READ_MODEL.to_string()])
    }

    fn fixture_dir_a10(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("r3-a10")
            .join(name)
    }
}
