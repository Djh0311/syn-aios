use crate::utils::fs_ops::remove_file_if_exists;
use crate::utils::hash::sha256_hex_bytes as sha256_hex;
use crate::workbench_sqlite_apply::{apply_fixture_dir_to_temp_db, table_count};
use crate::workbench_sqlite_exporter::{
    export_confirmed_db_to_json_dry_run, export_temp_db_to_json_dry_run,
    SqliteExportDryRunManifest, SqliteProjectedFile,
};
use crate::workbench_sqlite_importer::canonical_json_hash;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

const LEVEL_A: &str = "level_a_fixture";
const LEVEL_B_WORKBENCH_OWNED_STATE: &str = "level_b_workbench_owned_state";
const PRODUCTION_OBSERVATION_MODE: &str = "production_observation_export_verification";
const PRODUCTION_OBSERVATION_SCHEMA_VERSION: &str = "workbench_sqlite_production_observation.v1";
const LEVEL_A_OBSERVATION_MODE: &str = "level_a_fixture_temp";
const WORKFLOW_STATE_SUMMARY_READ_MODEL: &str = "workflow_state_summary";
const DEFAULT_PRODUCTION_OBSERVATION_DENIED_PATH_MARKERS: &[&str] = &[
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteProductionObservationFailurePoint {
    DbUnavailable,
    DbSchemaMismatch,
    DbIntegrityFailure,
    DbHashMismatch,
    FallbackHashMismatch,
    ExportHashMismatch,
    ProjectionFileMissing,
    ProjectionFileCorrupt,
    ObservationDriftBetweenSamples,
    RollbackManifestMissing,
    RollbackManifestIncomplete,
    AfterFirstSampleBeforeSecondSample,
    AfterFallbackSelectedBeforeReportCommit,
    AfterRollbackSelectedBeforeReportCommit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionObservationReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) level: String,
    pub(crate) status: String,
    pub(crate) observation_mode: String,
    pub(crate) read_model_name: String,
    pub(crate) feature_flag_enabled: bool,
    pub(crate) observation_source: String,
    pub(crate) fallback_decision: String,
    pub(crate) degraded: bool,
    pub(crate) db_path_hash: String,
    pub(crate) fallback_root_hash: String,
    pub(crate) projection_hash: String,
    pub(crate) export_manifest_hash: String,
    pub(crate) rollback_manifest_hash: String,
    pub(crate) expected_db_hash: Option<String>,
    pub(crate) actual_db_hash: Option<String>,
    pub(crate) expected_fallback_hash: Option<String>,
    pub(crate) actual_fallback_hash: String,
    pub(crate) record_counts: BTreeMap<String, usize>,
    pub(crate) samples: Vec<SqliteObservationSample>,
    pub(crate) export_verification: Option<SqliteExportVerification>,
    pub(crate) recovery_dry_run: SqliteObservationRollbackVerification,
    pub(crate) safety_flags: SqliteProductionObservationSafetyFlags,
    pub(crate) failure_point: Option<String>,
    pub(crate) redaction_policy: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionObservationSafetyFlags {
    pub(crate) production_observation_enabled: bool,
    pub(crate) product_global_read_path_changed: bool,
    pub(crate) app_startup_reads_db: bool,
    pub(crate) tauri_command_reads_db: bool,
    pub(crate) ui_reads_db: bool,
    pub(crate) stop_write_json: bool,
    pub(crate) source_json_written: bool,
    pub(crate) new_write_path_added: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) codex_home_touched: bool,
}

impl SqliteProductionObservationSafetyFlags {
    fn for_db_success() -> Self {
        Self {
            production_observation_enabled: true,
            product_global_read_path_changed: false,
            app_startup_reads_db: false,
            tauri_command_reads_db: false,
            ui_reads_db: false,
            stop_write_json: false,
            source_json_written: false,
            new_write_path_added: false,
            production_restore_performed: false,
            codex_home_touched: false,
        }
    }

    fn for_fallback() -> Self {
        Self {
            production_observation_enabled: false,
            product_global_read_path_changed: false,
            app_startup_reads_db: false,
            tauri_command_reads_db: false,
            ui_reads_db: false,
            stop_write_json: false,
            source_json_written: false,
            new_write_path_added: false,
            production_restore_performed: false,
            codex_home_touched: false,
        }
    }
}

struct ProductionObservationFallback {
    projection_hash: String,
    fallback_root_hash: String,
    export_manifest_hash: String,
    rollback_manifest_hash: String,
    counts: BTreeMap<String, usize>,
}

struct ProductionObservationDbRead {
    projection_hash: String,
    export_manifest_hash: String,
    rollback_manifest_hash: String,
    counts: BTreeMap<String, usize>,
    samples: Vec<SqliteObservationSample>,
    export_verification: SqliteExportVerification,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SqliteObservationLevelBConfig {
    pub(crate) confirmed_db_path: PathBuf,
    pub(crate) confirmed_fallback_root: PathBuf,
    pub(crate) confirmed_work_dir: PathBuf,
    pub(crate) confirmed_projection_root: PathBuf,
    pub(crate) confirmed_rollback_manifest_path: PathBuf,
    pub(crate) confirmed_observation_report_path: PathBuf,
}

pub(crate) fn rehearse_production_observation_level_a(
    observation_mode: &str,
    feature_flag_enabled: bool,
    read_model_name: &str,
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    observation_report_path: &Path,
    rollback_manifest_path: &Path,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    allowed_read_models: &BTreeSet<String>,
    denied_path_markers: &[String],
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<SqliteProductionObservationReport, String> {
    rehearse_production_observation(
        LEVEL_A,
        observation_mode,
        feature_flag_enabled,
        read_model_name,
        db_path,
        json_fallback_root,
        projection_root,
        observation_report_path,
        rollback_manifest_path,
        expected_db_hash,
        expected_fallback_hash,
        allowed_read_models,
        denied_path_markers,
        failure_point,
        |db, fallback, projection, rollback, report, denied| {
            validate_production_observation_paths(
                db, fallback, projection, rollback, report, denied,
            )
        },
        |db, projection, rollback, failure| {
            read_production_observation_db(db, projection, rollback, failure)
        },
    )
}

pub(crate) fn rehearse_production_observation_level_b_workbench_owned_state(
    observation_mode: &str,
    feature_flag_enabled: bool,
    read_model_name: &str,
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    observation_report_path: &Path,
    rollback_manifest_path: &Path,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    allowed_read_models: &BTreeSet<String>,
    denied_path_markers: &[String],
    config: &SqliteObservationLevelBConfig,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<SqliteProductionObservationReport, String> {
    rehearse_production_observation(
        LEVEL_B_WORKBENCH_OWNED_STATE,
        observation_mode,
        feature_flag_enabled,
        read_model_name,
        db_path,
        json_fallback_root,
        projection_root,
        observation_report_path,
        rollback_manifest_path,
        expected_db_hash,
        expected_fallback_hash,
        allowed_read_models,
        denied_path_markers,
        failure_point,
        |db, fallback, projection, rollback, report, denied| {
            validate_level_b_production_observation_paths(
                db, fallback, projection, rollback, report, config, denied,
            )
        },
        |db, projection, rollback, failure| {
            read_confirmed_production_observation_db(
                db,
                &config.confirmed_db_path,
                projection,
                rollback,
                failure,
            )
        },
    )
}

fn rehearse_production_observation(
    level: &str,
    observation_mode: &str,
    feature_flag_enabled: bool,
    read_model_name: &str,
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    observation_report_path: &Path,
    rollback_manifest_path: &Path,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    allowed_read_models: &BTreeSet<String>,
    denied_path_markers: &[String],
    failure_point: Option<SqliteProductionObservationFailurePoint>,
    validate_paths: impl Fn(&Path, &Path, &Path, &Path, &Path, &[String]) -> Result<(), String>,
    read_db: impl Fn(
        &Path,
        &Path,
        &Path,
        Option<SqliteProductionObservationFailurePoint>,
    ) -> Result<ProductionObservationDbRead, String>,
) -> Result<SqliteProductionObservationReport, String> {
    validate_production_observation_mode(observation_mode)?;
    validate_production_observation_read_model(read_model_name, allowed_read_models)?;
    let denied_markers = effective_production_observation_denied_markers(denied_path_markers);
    validate_paths(
        db_path,
        json_fallback_root,
        projection_root,
        rollback_manifest_path,
        observation_report_path,
        &denied_markers,
    )?;
    remove_file_if_exists(observation_report_path)?;

    let fallback = verified_production_observation_fallback(
        level,
        json_fallback_root,
        read_model_name,
        expected_fallback_hash,
        &denied_markers,
        failure_point,
    )?;

    if !feature_flag_enabled {
        let report = production_observation_report(
            level,
            "feature_flag_disabled_json_fallback_observation",
            observation_mode,
            read_model_name,
            false,
            "json_fallback",
            "feature_flag_disabled",
            true,
            db_path,
            &fallback.fallback_root_hash,
            &fallback.projection_hash,
            &fallback.export_manifest_hash,
            &fallback.rollback_manifest_hash,
            expected_db_hash,
            None,
            expected_fallback_hash,
            &fallback.projection_hash,
            fallback.counts,
            Vec::new(),
            None,
            SqliteProductionObservationSafetyFlags::for_fallback(),
            failure_point,
        );
        write_production_observation_report(observation_report_path, &report)?;
        return Ok(report);
    }

    if failure_point == Some(SqliteProductionObservationFailurePoint::DbUnavailable) {
        remove_file_if_exists(db_path)?;
    }
    let mut actual_db_hash = if db_path.exists() {
        Some(file_hash(db_path)?)
    } else {
        None
    };
    if failure_point == Some(SqliteProductionObservationFailurePoint::DbHashMismatch) {
        actual_db_hash = Some("injected_db_hash_mismatch".to_string());
    }
    if let (Some(expected), Some(actual)) = (expected_db_hash, actual_db_hash.as_deref()) {
        if actual != expected {
            remove_file_if_exists(observation_report_path)?;
            return Err(format!(
                "production_observation_blocked:db_hash_mismatch:expected={expected}:actual={}",
                actual
            ));
        }
    }

    if failure_point == Some(SqliteProductionObservationFailurePoint::DbSchemaMismatch) {
        remove_file_if_exists(db_path)?;
        let connection = Connection::open(db_path).map_err(|error| {
            format!(
                "create production observation schema mismatch db failed {}: {error}",
                db_path.display()
            )
        })?;
        connection
            .execute_batch("CREATE TABLE wrong_schema_marker (id TEXT PRIMARY KEY);")
            .map_err(|error| {
                format!(
                    "write production observation schema mismatch db failed {}: {error}",
                    db_path.display()
                )
            })?;
    }
    if failure_point == Some(SqliteProductionObservationFailurePoint::DbIntegrityFailure) {
        fs::write(db_path, b"not a sqlite database").map_err(|error| {
            format!(
                "write corrupt production observation db fixture failed {}: {error}",
                db_path.display()
            )
        })?;
    }

    match read_db(
        db_path,
        projection_root,
        rollback_manifest_path,
        failure_point,
    ) {
        Ok(db_observation) => {
            let report = production_observation_report(
                level,
                "stable_verified",
                observation_mode,
                read_model_name,
                true,
                "db_limited_observation",
                "not_used",
                false,
                db_path,
                &fallback.fallback_root_hash,
                &db_observation.projection_hash,
                &db_observation.export_manifest_hash,
                &db_observation.rollback_manifest_hash,
                expected_db_hash,
                actual_db_hash.as_deref(),
                expected_fallback_hash,
                &fallback.projection_hash,
                db_observation.counts,
                db_observation.samples,
                Some(db_observation.export_verification),
                if level == LEVEL_B_WORKBENCH_OWNED_STATE {
                    SqliteProductionObservationSafetyFlags::for_fallback()
                } else {
                    SqliteProductionObservationSafetyFlags::for_db_success()
                },
                failure_point,
            );
            write_production_observation_report(observation_report_path, &report)?;
            Ok(report)
        }
        Err(reason) if reason.starts_with("production_observation_fallback:") => {
            if failure_point
                == Some(SqliteProductionObservationFailurePoint::AfterFallbackSelectedBeforeReportCommit)
            {
                remove_file_if_exists(observation_report_path)?;
                return Err(
                    "injected_failure_after_fallback_selected_before_report_commit".to_string(),
                );
            }
            let report = production_observation_report(
                level,
                "fallback_degraded",
                observation_mode,
                read_model_name,
                true,
                "json_fallback",
                &format!("selected:{reason}"),
                true,
                db_path,
                &fallback.fallback_root_hash,
                &fallback.projection_hash,
                &fallback.export_manifest_hash,
                &fallback.rollback_manifest_hash,
                expected_db_hash,
                actual_db_hash.as_deref(),
                expected_fallback_hash,
                &fallback.projection_hash,
                fallback.counts,
                Vec::new(),
                None,
                SqliteProductionObservationSafetyFlags::for_fallback(),
                failure_point,
            );
            write_production_observation_report(observation_report_path, &report)?;
            Ok(report)
        }
        Err(reason) => {
            remove_file_if_exists(observation_report_path)?;
            Err(reason)
        }
    }
}

fn validate_production_observation_mode(observation_mode: &str) -> Result<(), String> {
    if observation_mode != LEVEL_A_OBSERVATION_MODE {
        return Err(format!(
            "production_observation_blocked:unsupported_observation_mode:{observation_mode}"
        ));
    }
    Ok(())
}

fn validate_production_observation_read_model(
    read_model_name: &str,
    allowed_read_models: &BTreeSet<String>,
) -> Result<(), String> {
    if read_model_name != WORKFLOW_STATE_SUMMARY_READ_MODEL {
        return Err(format!(
            "production_observation_blocked:unsupported_read_model:{read_model_name}"
        ));
    }
    if !allowed_read_models.contains(read_model_name) {
        return Err(format!(
            "production_observation_blocked:read_model_not_allowed:{read_model_name}"
        ));
    }
    Ok(())
}

fn effective_production_observation_denied_markers(markers: &[String]) -> Vec<String> {
    let mut values = DEFAULT_PRODUCTION_OBSERVATION_DENIED_PATH_MARKERS
        .iter()
        .map(|marker| (*marker).to_string())
        .collect::<Vec<_>>();
    values.extend(markers.iter().cloned());
    values.sort();
    values.dedup();
    values
}

fn validate_production_observation_paths(
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    observation_report_path: &Path,
    denied_markers: &[String],
) -> Result<(), String> {
    for path in [
        db_path,
        json_fallback_root,
        projection_root,
        rollback_manifest_path,
        observation_report_path,
    ] {
        if !path.is_absolute() {
            return Err(format!(
                "production_observation_blocked:absolute_path_required:{}",
                path.display()
            ));
        }
        reject_production_denied_path_markers(path, denied_markers)?;
    }
    validate_temp_db_path(db_path)?;
    if !json_fallback_root.starts_with(std::env::temp_dir())
        && !json_fallback_root.starts_with(manifest_r3_a11_fixture_root())
    {
        return Err(format!(
            "production_observation_blocked:fallback_root_must_be_temp_or_r3_a11_fixture:{}",
            json_fallback_root.display()
        ));
    }
    let projection_allowed = projection_root.starts_with(std::env::temp_dir())
        || projection_root.starts_with(manifest_r3_a11_fixture_root());
    if !projection_allowed
        || !rollback_manifest_path.starts_with(projection_root)
        || !observation_report_path.starts_with(projection_root)
    {
        return Err(format!(
            "production_observation_blocked:projection_manifest_report_must_be_temp_or_r3_a11_fixture:{}:{}:{}",
            projection_root.display(),
            rollback_manifest_path.display(),
            observation_report_path.display()
        ));
    }
    if projection_root.starts_with(json_fallback_root)
        || json_fallback_root.starts_with(projection_root)
    {
        return Err(
            "production_observation_blocked:fallback_and_projection_roots_must_be_separate"
                .to_string(),
        );
    }
    Ok(())
}

#[rustfmt::skip]
fn validate_level_b_production_observation_paths(
    db_path: &Path,
    json_fallback_root: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    observation_report_path: &Path,
    config: &SqliteObservationLevelBConfig,
    denied_markers: &[String],
) -> Result<(), String> {
    let prelude = |label: &str, actual: &Path, confirmed: Option<&Path>| -> Result<(), String> {
        if let Some(confirmed) = confirmed {
            if actual != confirmed {
                return Err(format!("production_observation_level_b_confirmed_path_mismatch:{label}:expected={}:actual={}", confirmed.display(), actual.display()));
            }
        }
        if !actual.is_absolute() {
            return Err(format!("production_observation_blocked:absolute_path_required:{}", actual.display()));
        }
        if actual.components().any(|component| matches!(component, Component::ParentDir | Component::CurDir)) {
            return Err(format!("production_observation_level_b_confirmed_path_must_be_clean:{label}:{}", actual.display()));
        }
        reject_production_denied_path_markers(actual, denied_markers)
    };
    let canonical_dir = |path: &Path, label: &str| -> Result<PathBuf, String> {
        fs::canonicalize(path).map_err(|error| format!("production_observation_level_b_canonicalize_failed:{label}:{}:{error}", path.display()))
            .and_then(|canonical| canonical.is_dir().then_some(canonical).ok_or_else(|| format!("production_observation_level_b_dir_required:{label}:{}", path.display())))
    };
    let existing = |label: &str, actual: &Path, confirmed: &Path, file_required: bool| -> Result<PathBuf, String> {
        prelude(label, actual, Some(confirmed))?;
        let actual_canonical = fs::canonicalize(actual).map_err(|error| format!("production_observation_level_b_canonicalize_failed:{label}:{}:{error}", actual.display()))?;
        let confirmed_canonical = fs::canonicalize(confirmed).map_err(|error| format!("production_observation_level_b_canonicalize_failed:confirmed_{label}:{}:{error}", confirmed.display()))?;
        if actual_canonical != confirmed_canonical || actual != actual_canonical.as_path() { return Err(format!("production_observation_level_b_confirmed_path_must_be_canonical:{label}:expected={}:actual={}", confirmed_canonical.display(), actual.display())); }
        match (file_required, actual_canonical.is_file(), actual_canonical.is_dir()) {
            (true, false, _) => Err(format!("production_observation_level_b_file_required:{label}:{}", actual.display())),
            (false, _, false) => Err(format!("production_observation_level_b_dir_required:{label}:{}", actual.display())),
            _ => Ok(actual_canonical),
        }
    };
    let work_dir = |work_dir: &Path, fallback_root: &Path| -> Result<PathBuf, String> {
        prelude("work_dir", work_dir, None)?;
        let canonical_work_dir = canonical_dir(work_dir, "work_dir")?;
        if work_dir != canonical_work_dir.as_path() {
            return Err(format!("production_observation_level_b_confirmed_path_must_be_canonical:work_dir:expected={}:actual={}", canonical_work_dir.display(), work_dir.display()));
        }
        let canonical_fallback = canonical_dir(fallback_root, "fallback_root")?;
        if canonical_work_dir.starts_with(&canonical_fallback) {
            return Err(format!("production_observation_blocked:path_inside_fallback_root_denied:{}", work_dir.display()));
        }
        Ok(canonical_work_dir)
    };
    let output = |label: &str, actual: &Path, confirmed: &Path, work_dir: &Path, fallback_root: &Path, db_path: &Path| -> Result<(), String> {
        prelude(label, actual, Some(confirmed))?;
        let canonical_actual = canonicalize_existing_or_parent_observation(actual, label)?;
        let canonical_confirmed = canonicalize_existing_or_parent_observation(confirmed, label)?;
        let canonical_work_dir = canonical_dir(work_dir, "work_dir")?;
        let canonical_db_dir = db_path.parent().ok_or_else(|| format!("production_observation_level_b_db_parent_required:{}", db_path.display())).and_then(|parent| canonical_dir(parent, "db_parent"))?;
        if canonical_actual != canonical_confirmed || actual != canonical_actual.as_path() { return Err(format!("production_observation_level_b_confirmed_path_must_be_canonical:{label}:expected={}:actual={}", canonical_confirmed.display(), actual.display())); }
        if canonical_actual.starts_with(fallback_root) { return Err(format!("production_observation_blocked:path_inside_fallback_root_denied:{}", actual.display())); }
        if canonical_actual.starts_with(&canonical_db_dir) { return Err(format!("production_observation_blocked:path_inside_db_dir_denied:{}", actual.display())); }
        if !canonical_actual.starts_with(&canonical_work_dir) { return Err(format!("production_observation_blocked:path_outside_confirmed_work_dir:{label}:{}", actual.display())); }
        Ok(())
    };
    let _db = existing("db_path", db_path, &config.confirmed_db_path, true)?;
    let fallback = existing("fallback_root", json_fallback_root, &config.confirmed_fallback_root, false)?;
    let work = work_dir(&config.confirmed_work_dir, &fallback)?;
    for (label, actual, confirmed) in [("projection_root", projection_root, &config.confirmed_projection_root), ("rollback_manifest_path", rollback_manifest_path, &config.confirmed_rollback_manifest_path), ("observation_report_path", observation_report_path, &config.confirmed_observation_report_path)] {
        output(label, actual, confirmed, &work, &fallback, db_path)?;
    }
    Ok(())
}

fn verified_production_observation_fallback(
    level: &str,
    fallback_root: &Path,
    read_model_name: &str,
    expected_fallback_hash: Option<&str>,
    denied_markers: &[String],
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<ProductionObservationFallback, String> {
    reject_production_denied_path_markers(fallback_root, denied_markers)?;
    let workflow = load_production_workflow_state_from_root(fallback_root)?;
    ensure_no_forbidden_production_value(&workflow)?;
    let summary = workflow_state_summary(&workflow)?;
    let mut projection_hash = canonical_json_hash(&summary);
    if failure_point == Some(SqliteProductionObservationFailurePoint::FallbackHashMismatch) {
        projection_hash = "injected_fallback_hash_mismatch".to_string();
    }
    if let Some(expected) = expected_fallback_hash {
        if projection_hash != expected {
            return Err(format!(
                "production_observation_blocked:fallback_hash_mismatch:expected={expected}:actual={projection_hash}"
            ));
        }
    }
    let fallback_root_hash = dir_hash(fallback_root)?;
    let counts = summary_counts(&summary)?;
    let export_manifest = json!({
        "schema_version": PRODUCTION_OBSERVATION_SCHEMA_VERSION,
        "mode": PRODUCTION_OBSERVATION_MODE,
        "level": level,
        "status": "verified_json_fallback",
        "read_model_name": read_model_name,
        "fallback_root_hash": fallback_root_hash,
        "projection_hash": projection_hash,
        "counts": counts,
        "redaction_policy": production_redaction_policy(),
        "runtime_log_alias_policy": "canonical_runtime_logs_only"
    });
    let rollback_manifest = json!({
        "schema_version": PRODUCTION_OBSERVATION_SCHEMA_VERSION,
        "mode": PRODUCTION_OBSERVATION_MODE,
        "level": level,
        "status": "fallback_recovery_dry_run_only",
        "projection_hash": projection_hash,
        "recovery_dry_run": production_observation_recovery_plan("json_fallback", &projection_hash)
    });
    Ok(ProductionObservationFallback {
        projection_hash,
        fallback_root_hash,
        export_manifest_hash: canonical_json_hash(&export_manifest),
        rollback_manifest_hash: canonical_json_hash(&rollback_manifest),
        counts,
    })
}

fn read_production_observation_db(
    db_path: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<ProductionObservationDbRead, String> {
    read_production_observation_db_with(
        LEVEL_A,
        db_path,
        projection_root,
        rollback_manifest_path,
        failure_point,
        |label| observation_sample(db_path, projection_root, label),
        || write_projection_files(db_path, projection_root),
    )
}

fn read_confirmed_production_observation_db(
    db_path: &Path,
    confirmed_db_path: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<ProductionObservationDbRead, String> {
    read_production_observation_db_with(
        LEVEL_B_WORKBENCH_OWNED_STATE,
        db_path,
        projection_root,
        rollback_manifest_path,
        failure_point,
        |label| observation_sample_confirmed(db_path, confirmed_db_path, projection_root, label),
        || write_confirmed_projection_files(db_path, confirmed_db_path, projection_root),
    )
}

fn read_production_observation_db_with(
    level: &str,
    db_path: &Path,
    projection_root: &Path,
    rollback_manifest_path: &Path,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
    sample: impl Fn(&str) -> Result<SqliteObservationSample, String>,
    write_projection: impl Fn() -> Result<(), String>,
) -> Result<ProductionObservationDbRead, String> {
    verify_production_db_integrity(db_path)
        .map_err(|reason| format!("production_observation_fallback:{reason}"))?;
    let sample_one = sample("sample_1").map_err(production_observation_error)?;
    if failure_point
        == Some(SqliteProductionObservationFailurePoint::AfterFirstSampleBeforeSecondSample)
    {
        return Err("injected_failure_after_first_sample_before_second_sample".to_string());
    }
    let mut sample_two = sample("sample_2").map_err(production_observation_error)?;
    if failure_point
        == Some(SqliteProductionObservationFailurePoint::ObservationDriftBetweenSamples)
    {
        sample_two.projection_hash = "injected_production_observation_drift".to_string();
    }
    verify_sample_stability(&sample_one, &sample_two).map_err(production_observation_error)?;
    let db_hash = file_hash(db_path)?;
    let mut export_verification = export_verification(&db_hash, &sample_two);
    if failure_point == Some(SqliteProductionObservationFailurePoint::ExportHashMismatch) {
        export_verification.db_export_hash = "injected_export_hash_mismatch".to_string();
    }
    verify_export_hashes(&export_verification, &sample_two)
        .map_err(production_observation_error)?;

    write_projection().map_err(production_observation_error)?;
    if failure_point == Some(SqliteProductionObservationFailurePoint::ProjectionFileMissing) {
        if let Some(file) = sample_two.projected_files.first() {
            remove_file_if_exists(&projection_root.join(&file.path))?;
        }
    }
    if failure_point == Some(SqliteProductionObservationFailurePoint::ProjectionFileCorrupt) {
        if let Some(file) = sample_two.projected_files.first() {
            fs::write(projection_root.join(&file.path), b"{corrupt").map_err(|error| {
                format!(
                    "write corrupt production observation projection fixture failed {}: {error}",
                    file.path
                )
            })?;
        }
    }
    verify_projection_files(projection_root, &sample_two.projected_files)
        .map_err(production_observation_error)?;

    let manifest_hash = write_production_observation_rollback_manifest(
        level,
        rollback_manifest_path,
        db_path,
        projection_root,
        &sample_two,
    )?;
    let verified_manifest_hash =
        verify_production_observation_rollback_manifest(rollback_manifest_path, failure_point)?;
    if manifest_hash != verified_manifest_hash {
        return Err("production_observation_blocked:rollback_manifest_hash_mismatch".to_string());
    }
    if failure_point
        == Some(SqliteProductionObservationFailurePoint::AfterRollbackSelectedBeforeReportCommit)
    {
        return Err("injected_failure_after_rollback_selected_before_report_commit".to_string());
    }

    Ok(ProductionObservationDbRead {
        projection_hash: sample_two.projection_hash.clone(),
        export_manifest_hash: export_verification.export_manifest_hash.clone(),
        rollback_manifest_hash: verified_manifest_hash,
        counts: record_counts_from_db_rows(&sample_two.db_row_counts)?,
        samples: vec![sample_one, sample_two],
        export_verification,
    })
}

fn verify_production_db_integrity(db_path: &Path) -> Result<(), String> {
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

fn production_observation_report(
    level: &str,
    status: &str,
    observation_mode: &str,
    read_model_name: &str,
    feature_flag_enabled: bool,
    observation_source: &str,
    fallback_decision: &str,
    degraded: bool,
    db_path: &Path,
    fallback_root_hash: &str,
    projection_hash: &str,
    export_manifest_hash: &str,
    rollback_manifest_hash: &str,
    expected_db_hash: Option<&str>,
    actual_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    actual_fallback_hash: &str,
    record_counts: BTreeMap<String, usize>,
    samples: Vec<SqliteObservationSample>,
    export_verification: Option<SqliteExportVerification>,
    safety_flags: SqliteProductionObservationSafetyFlags,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> SqliteProductionObservationReport {
    SqliteProductionObservationReport {
        schema_version: PRODUCTION_OBSERVATION_SCHEMA_VERSION.to_string(),
        mode: PRODUCTION_OBSERVATION_MODE.to_string(),
        level: level.to_string(),
        status: status.to_string(),
        observation_mode: observation_mode.to_string(),
        read_model_name: read_model_name.to_string(),
        feature_flag_enabled,
        observation_source: observation_source.to_string(),
        fallback_decision: fallback_decision.to_string(),
        degraded,
        db_path_hash: path_hash(db_path),
        fallback_root_hash: fallback_root_hash.to_string(),
        projection_hash: projection_hash.to_string(),
        export_manifest_hash: export_manifest_hash.to_string(),
        rollback_manifest_hash: rollback_manifest_hash.to_string(),
        expected_db_hash: expected_db_hash.map(ToString::to_string),
        actual_db_hash: actual_db_hash.map(ToString::to_string),
        expected_fallback_hash: expected_fallback_hash.map(ToString::to_string),
        actual_fallback_hash: actual_fallback_hash.to_string(),
        record_counts,
        samples,
        export_verification,
        recovery_dry_run: production_observation_recovery_plan(observation_source, projection_hash),
        safety_flags,
        failure_point: failure_point.map(|point| format!("{point:?}")),
        redaction_policy: production_redaction_policy(),
    }
}

fn write_production_observation_report(
    path: &Path,
    report: &SqliteProductionObservationReport,
) -> Result<(), String> {
    if report.status != "stable_verified"
        && report.status != "fallback_degraded"
        && report.status != "feature_flag_disabled_json_fallback_observation"
    {
        return Err(format!(
            "production_observation_report_status_not_committable:{}",
            report.status
        ));
    }
    if report.safety_flags.product_global_read_path_changed
        || report.safety_flags.app_startup_reads_db
        || report.safety_flags.tauri_command_reads_db
        || report.safety_flags.ui_reads_db
        || report.safety_flags.stop_write_json
        || report.safety_flags.source_json_written
        || report.safety_flags.new_write_path_added
        || report.safety_flags.production_restore_performed
        || report.safety_flags.codex_home_touched
    {
        return Err("production_observation_forbidden_safety_flag_true".to_string());
    }
    if report.status == "stable_verified"
        && report.level != LEVEL_B_WORKBENCH_OWNED_STATE
        && !report.safety_flags.production_observation_enabled
    {
        return Err("production_observation_stable_report_requires_enabled_flag".to_string());
    }
    if report.status != "stable_verified" && report.safety_flags.production_observation_enabled {
        return Err("production_observation_fallback_report_must_not_enable_db".to_string());
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize production observation report failed: {error}"))?,
    )
}

fn write_production_observation_rollback_manifest(
    level: &str,
    path: &Path,
    db_path: &Path,
    projection_root: &Path,
    sample: &SqliteObservationSample,
) -> Result<String, String> {
    let recovery_dry_run =
        production_observation_recovery_plan("rollback_manifest", &sample.projection_hash);
    let payload = json!({
        "schema_version": PRODUCTION_OBSERVATION_SCHEMA_VERSION,
        "mode": PRODUCTION_OBSERVATION_MODE,
        "level": level,
        "status": "completed",
        "db_path_hash": path_hash(db_path),
        "projection_root_hash": path_hash(projection_root),
        "projection_hash": sample.projection_hash,
        "export_hash": sample.export_hash,
        "projected_files": sample.projected_files,
        "record_counts": record_counts_from_db_rows(&sample.db_row_counts)?,
        "redaction_policy": sample.redaction_policy,
        "runtime_log_alias_policy": "canonical_runtime_logs_only",
        "recovery_dry_run": recovery_dry_run
    });
    let manifest_hash = canonical_json_hash(&payload);
    let mut manifest = payload;
    manifest["rollback_manifest_hash"] = Value::String(manifest_hash.clone());
    commit_rollback_manifest(path, &manifest)?;
    Ok(manifest_hash)
}

fn verify_production_observation_rollback_manifest(
    path: &Path,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<String, String> {
    if failure_point == Some(SqliteProductionObservationFailurePoint::RollbackManifestMissing) {
        remove_file_if_exists(path)?;
    }
    if failure_point == Some(SqliteProductionObservationFailurePoint::RollbackManifestIncomplete) {
        write_json_file(
            path,
            &json!({
                "schema_version": PRODUCTION_OBSERVATION_SCHEMA_VERSION,
                "mode": PRODUCTION_OBSERVATION_MODE,
                "level": LEVEL_A,
                "status": "manifest_commit_failed_before_complete",
                "production_restore_performed": false
            }),
        )?;
    }
    let bytes = fs::read(path).map_err(|error| {
        format!("production_observation_blocked:rollback_manifest_missing:{error}")
    })?;
    let manifest: Value = serde_json::from_slice(&bytes).map_err(|error| {
        format!("production_observation_blocked:rollback_manifest_corrupt:{error}")
    })?;
    if manifest.get("status").and_then(Value::as_str) != Some("completed") {
        remove_file_if_exists(path)?;
        return Err("production_observation_blocked:rollback_manifest_incomplete".to_string());
    }
    let restore_performed = manifest
        .get("recovery_dry_run")
        .and_then(|value| value.get("production_restore_performed"))
        .and_then(Value::as_bool);
    if restore_performed != Some(false) {
        return Err("production_observation_blocked:rollback_manifest_not_dry_run".to_string());
    }
    manifest
        .get("rollback_manifest_hash")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| "production_observation_blocked:rollback_manifest_hash_missing".to_string())
}

fn production_observation_recovery_plan(
    reason: &str,
    projection_hash: &str,
) -> SqliteObservationRollbackVerification {
    SqliteObservationRollbackVerification {
        status: "production_observation_recovery_dry_run_only".to_string(),
        would_disable_db_read_cut: true,
        would_use_last_verified_json_projection: true,
        would_preserve_db_for_audit: true,
        would_require_supervisor_decision: true,
        production_restore_performed: false,
        selected_projection_hash: projection_hash.to_string(),
        instructions: vec![
            format!("would disable limited read-cut and production observation before recovery: {reason}"),
            "would use verified JSON fallback or the last verified JSON projection".to_string(),
            "would preserve SQLite DB for audit".to_string(),
            "would require supervisor decision before any production restore".to_string(),
            "production restore is not performed by this production observation rehearsal"
                .to_string(),
        ],
    }
}

fn load_production_workflow_state_from_root(root: &Path) -> Result<Value, String> {
    let path = root.join("workflow-state.v0.json");
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "production_observation_blocked:fallback_workflow_state_missing:{}:{error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "production_observation_blocked:fallback_workflow_state_corrupt:{}:{error}",
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
            .ok_or_else(|| format!("production_observation_blocked:summary_count_missing:{key}"))?;
        counts.insert(key.to_string(), value as usize);
    }
    Ok(counts)
}

fn array_count(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn record_counts_from_db_rows(
    counts: &BTreeMap<String, i64>,
) -> Result<BTreeMap<String, usize>, String> {
    let mut result = BTreeMap::new();
    for (key, value) in counts {
        let value = usize::try_from(*value).map_err(|_| {
            format!("production_observation_blocked:negative_record_count:{key}:{value}")
        })?;
        result.insert(key.clone(), value);
    }
    Ok(result)
}

fn ensure_no_forbidden_production_value(value: &Value) -> Result<(), String> {
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
                "production_observation_blocked:forbidden_projection_body:{marker}"
            ));
        }
    }
    Ok(())
}

fn production_observation_error(reason: String) -> String {
    if reason.starts_with("production_observation_") {
        reason
    } else if let Some(suffix) = reason.strip_prefix("observation_blocked:") {
        format!("production_observation_blocked:{suffix}")
    } else if let Some(suffix) = reason.strip_prefix("observation_degraded:") {
        format!("production_observation_fallback:{suffix}")
    } else {
        format!("production_observation_blocked:{reason}")
    }
}

fn reject_production_denied_path_markers(
    path: &Path,
    denied_markers: &[String],
) -> Result<(), String> {
    let normalized = path.display().to_string().to_ascii_lowercase();
    for marker in denied_markers {
        if normalized.contains(&marker.to_ascii_lowercase()) {
            return Err(format!(
                "production_observation_blocked:denied_path_marker:{}:{}",
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
            "production_observation_blocked:file_hash_read_failed:{}:{error}",
            path.display()
        )
    })?;
    Ok(sha256_hex(&bytes))
}

fn dir_hash(root: &Path) -> Result<String, String> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| {
            format!(
                "production_observation_blocked:read_dir_for_hash_failed:{}:{error}",
                root.display()
            )
        })?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "production_observation_blocked:read_dir_entry_for_hash_failed:{}:{error}",
                root.display()
            )
        })?;
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

fn canonicalize_existing_or_parent_observation(
    path: &Path,
    label: &str,
) -> Result<PathBuf, String> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|error| {
            format!(
                "production_observation_level_b_canonicalize_failed:{label}:{}:{error}",
                path.display()
            )
        });
    }
    let mut ancestor = path.parent().ok_or_else(|| {
        format!(
            "production_observation_level_b_parent_required:{label}:{}",
            path.display()
        )
    })?;
    while !ancestor.exists() {
        ancestor = ancestor.parent().ok_or_else(|| {
            format!(
                "production_observation_level_b_existing_parent_required:{label}:{}",
                path.display()
            )
        })?;
    }
    let canonical_ancestor = fs::canonicalize(ancestor).map_err(|error| {
        format!(
            "production_observation_level_b_parent_canonicalize_failed:{label}:{}:{error}",
            ancestor.display()
        )
    })?;
    let suffix = path.strip_prefix(ancestor).map_err(|error| {
        format!(
            "production_observation_level_b_suffix_failed:{label}:{}:{error}",
            path.display()
        )
    })?;
    Ok(canonical_ancestor.join(suffix))
}

fn production_redaction_policy() -> Vec<String> {
    vec![
        "prompt_body:omitted".to_string(),
        "full_transcript:omitted".to_string(),
        "secret_token_credential_keychain_oauth_provider_credential:omitted".to_string(),
        "rollout_body:omitted".to_string(),
    ]
}

fn manifest_r3_a11_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a11")
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
    observation_sample_from_manifest(db_path, label, manifest)
}

fn observation_sample_confirmed(
    db_path: &Path,
    confirmed_db_path: &Path,
    projection_root: &Path,
    label: &str,
) -> Result<SqliteObservationSample, String> {
    verify_db_integrity(db_path)?;
    let manifest = export_confirmed_db_to_json_dry_run(
        db_path,
        confirmed_db_path,
        &projection_root.display().to_string(),
    )?;
    observation_sample_from_manifest(db_path, label, manifest)
}

fn observation_sample_from_manifest(
    db_path: &Path,
    label: &str,
    manifest: SqliteExportDryRunManifest,
) -> Result<SqliteObservationSample, String> {
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
    write_projection_files_from_manifest(projection_root, manifest)
}

fn write_confirmed_projection_files(
    db_path: &Path,
    confirmed_db_path: &Path,
    projection_root: &Path,
) -> Result<(), String> {
    let manifest = export_confirmed_db_to_json_dry_run(
        db_path,
        confirmed_db_path,
        &projection_root.display().to_string(),
    )?;
    write_projection_files_from_manifest(projection_root, manifest)
}

fn write_projection_files_from_manifest(
    projection_root: &Path,
    manifest: SqliteExportDryRunManifest,
) -> Result<(), String> {
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
mod tests;
