use crate::workbench_sqlite_apply::{apply_fixture_dir_to_temp_db, table_count};
use crate::workbench_sqlite_exporter::{export_temp_db_to_json_dry_run, SqliteProjectedFile};
use crate::workbench_sqlite_importer::{
    canonical_json_hash, OPTIONAL_SIDECARS, PRIMARY_WORKFLOW_STATE,
};
use crate::workbench_sqlite_preflight::{
    scan_workbench_state_root_preflight_with_config, SqliteProductionPreflightConfig,
    SqliteProductionPreflightReport,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const MODE: &str = "copied_snapshot_apply";
const LEVEL_A: &str = "level_a_fixture";
const SNAPSHOT_APPLY_SCHEMA_VERSION: &str = "workbench_sqlite_copied_snapshot_apply.v1";
const COPY_MANIFEST_NAME: &str = "copied-snapshot-manifest.json";
const ROLLBACK_MANIFEST_NAME: &str = "rollback-boundary-manifest.json";
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
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SqliteSnapshotApplyFailurePoint {
    CopyInterruptedBeforeManifest,
    ApplyRejectedCorruptSnapshot,
    ExportHashMismatch,
    RollbackManifestMissing,
    RollbackManifestIncomplete,
    CleanupFailureAfterRollbackBoundary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SqliteSnapshotApplyConfig {
    pub(crate) allowed_sidecars: BTreeSet<String>,
    pub(crate) denied_path_markers: Vec<String>,
    pub(crate) expected_source_root_hash: Option<String>,
    pub(crate) expected_preflight_report_hash: Option<String>,
}

impl Default for SqliteSnapshotApplyConfig {
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
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotApplyReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) level: String,
    pub(crate) status: String,
    pub(crate) source_root_ref: String,
    pub(crate) source_root_hash: String,
    pub(crate) source_root_path_hash: String,
    pub(crate) snapshot_copy_root_ref: String,
    pub(crate) snapshot_copy_root_hash: String,
    pub(crate) snapshot_copy_root_path_hash: String,
    pub(crate) temp_db_path_hash: String,
    pub(crate) temp_export_root_hash: String,
    pub(crate) temp_export_root_path_hash: String,
    pub(crate) report_path_hash: String,
    pub(crate) copied_file_manifest: Vec<SqliteCopiedSnapshotFile>,
    pub(crate) preflight: SqliteSnapshotPreflightSummary,
    pub(crate) apply_summary: SqliteSnapshotApplySummary,
    pub(crate) export_verification: SqliteSnapshotExportVerification,
    pub(crate) rollback_boundary: SqliteSnapshotRollbackBoundary,
    pub(crate) flags: SqliteSnapshotSafetyFlags,
    pub(crate) failure_point: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteCopiedSnapshotFile {
    pub(crate) path_ref: String,
    pub(crate) path_hash: String,
    pub(crate) file_hash: String,
    pub(crate) size_bytes: u64,
    pub(crate) schema_version: Option<String>,
    pub(crate) revision: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotPreflightSummary {
    pub(crate) source_status: String,
    pub(crate) copy_status: String,
    pub(crate) source_report_hash: String,
    pub(crate) copy_report_hash: String,
    pub(crate) source_files_accepted: usize,
    pub(crate) copy_files_accepted: usize,
    pub(crate) source_blocked_reasons: usize,
    pub(crate) copy_blocked_reasons: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotApplySummary {
    pub(crate) status: String,
    pub(crate) batch_id: String,
    pub(crate) source_root_hash: String,
    pub(crate) records_inserted: usize,
    pub(crate) records_skipped: usize,
    pub(crate) sources_inserted: usize,
    pub(crate) db_row_counts: BTreeMap<String, i64>,
    pub(crate) runtime_log_alias_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotExportVerification {
    pub(crate) status: String,
    pub(crate) db_export_hash: String,
    pub(crate) projection_hash: String,
    pub(crate) export_manifest_hash: String,
    pub(crate) runtime_log_alias_policy: String,
    pub(crate) projected_files: Vec<SqliteSnapshotProjectedFile>,
    pub(crate) redaction_manifest: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotProjectedFile {
    pub(crate) path: String,
    pub(crate) projected_hash: String,
    pub(crate) record_count: usize,
    pub(crate) redaction_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotRollbackBoundary {
    pub(crate) status: String,
    pub(crate) rollback_manifest_hash: String,
    pub(crate) rollback_manifest_path_hash: String,
    pub(crate) would_disable_db_read_cut: bool,
    pub(crate) would_use_snapshot_projection: bool,
    pub(crate) would_preserve_temp_db_for_audit: bool,
    pub(crate) would_require_supervisor_decision: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) instructions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteSnapshotSafetyFlags {
    pub(crate) production_db_created: bool,
    pub(crate) production_root_written: bool,
    pub(crate) production_apply_performed: bool,
    pub(crate) read_cut_enabled: bool,
    pub(crate) stop_write_json: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) codex_home_touched: bool,
}

impl Default for SqliteSnapshotSafetyFlags {
    fn default() -> Self {
        Self {
            production_db_created: false,
            production_root_written: false,
            production_apply_performed: false,
            read_cut_enabled: false,
            stop_write_json: false,
            production_restore_performed: false,
            codex_home_touched: false,
        }
    }
}

pub(crate) fn rehearse_copied_snapshot_apply_level_a(
    source_snapshot_root: &Path,
    snapshot_copy_root: &Path,
    temp_db_path: &Path,
    temp_export_root: &Path,
    report_path: &Path,
    config: &SqliteSnapshotApplyConfig,
    failure_point: Option<SqliteSnapshotApplyFailurePoint>,
) -> Result<SqliteSnapshotApplyReport, String> {
    let denied_path_markers = effective_denied_path_markers(config);
    validate_level_a_source_snapshot_root(source_snapshot_root, &denied_path_markers)?;
    validate_temp_roots(
        source_snapshot_root,
        snapshot_copy_root,
        temp_db_path,
        temp_export_root,
        report_path,
        &denied_path_markers,
    )?;
    remove_file_if_exists(report_path)?;

    let preflight_config = preflight_config(config, &denied_path_markers);
    let source_preflight = scan_workbench_state_root_preflight_with_config(
        source_snapshot_root,
        None,
        &preflight_config,
    )?;
    ensure_preflight_ready("source", &source_preflight)?;
    if let Some(expected) = &config.expected_source_root_hash {
        if &source_preflight.source_root_hash != expected {
            return Err(format!(
                "snapshot_apply_blocked:source_root_hash_mismatch:expected={expected}:actual={}",
                source_preflight.source_root_hash
            ));
        }
    }
    let source_report_hash = report_hash(&source_preflight)?;
    if let Some(expected) = &config.expected_preflight_report_hash {
        if &source_report_hash != expected {
            return Err(format!(
                "snapshot_apply_blocked:preflight_report_hash_mismatch:expected={expected}:actual={source_report_hash}"
            ));
        }
    }

    reset_dir(snapshot_copy_root)?;
    reset_dir(temp_export_root)?;
    remove_file_if_exists(temp_db_path)?;
    let copied_file_manifest = copy_snapshot_files(
        source_snapshot_root,
        snapshot_copy_root,
        config,
        failure_point,
    )?;
    let copy_preflight = scan_workbench_state_root_preflight_with_config(
        snapshot_copy_root,
        None,
        &preflight_config,
    )?;
    ensure_preflight_ready("copy", &copy_preflight)?;
    if copy_preflight.source_root_hash != source_preflight.source_root_hash {
        return Err(format!(
            "snapshot_apply_blocked:copy_root_hash_mismatch:source={}:copy={}",
            source_preflight.source_root_hash, copy_preflight.source_root_hash
        ));
    }

    if failure_point == Some(SqliteSnapshotApplyFailurePoint::ApplyRejectedCorruptSnapshot) {
        fs::write(snapshot_copy_root.join(PRIMARY_WORKFLOW_STATE), b"{corrupt").map_err(
            |error| {
                format!(
                    "write corrupt copied snapshot failure fixture failed {}: {error}",
                    snapshot_copy_root.join(PRIMARY_WORKFLOW_STATE).display()
                )
            },
        )?;
    }
    let apply_report = apply_fixture_dir_to_temp_db(snapshot_copy_root, temp_db_path, None)?;
    let apply_summary = apply_summary(&apply_report, temp_db_path)?;
    let mut export_verification = export_verification(temp_db_path, temp_export_root)?;
    if failure_point == Some(SqliteSnapshotApplyFailurePoint::ExportHashMismatch) {
        export_verification.db_export_hash = "injected_export_hash_mismatch".to_string();
    }
    verify_export_hashes(&export_verification)?;
    write_export_projection(
        temp_db_path,
        temp_export_root,
        &export_verification.projected_files,
    )?;

    let mut rollback_boundary = rollback_boundary(temp_export_root, temp_db_path)?;
    write_rollback_manifest(temp_export_root, &rollback_boundary)?;
    if failure_point == Some(SqliteSnapshotApplyFailurePoint::RollbackManifestMissing) {
        remove_file_if_exists(&rollback_manifest_path(temp_export_root))?;
    }
    if failure_point == Some(SqliteSnapshotApplyFailurePoint::RollbackManifestIncomplete) {
        write_json_file(
            &rollback_manifest_path(temp_export_root),
            &json!({
                "schema_version": SNAPSHOT_APPLY_SCHEMA_VERSION,
                "status": "manifest_commit_failed_before_complete",
                "failure_point": "RollbackManifestIncomplete",
                "production_restore_performed": false
            }),
        )?;
    }
    rollback_boundary = verify_rollback_manifest(temp_export_root)?;

    if failure_point == Some(SqliteSnapshotApplyFailurePoint::CleanupFailureAfterRollbackBoundary) {
        rollback_boundary.instructions.push(
            "cleanup failure injected after rollback boundary; temp artifacts preserved for audit"
                .to_string(),
        );
    }
    write_copy_manifest(snapshot_copy_root, &copied_file_manifest)?;

    let report = SqliteSnapshotApplyReport {
        schema_version: SNAPSHOT_APPLY_SCHEMA_VERSION.to_string(),
        mode: MODE.to_string(),
        level: LEVEL_A.to_string(),
        status: "completed".to_string(),
        source_root_ref: source_snapshot_root.display().to_string(),
        source_root_hash: source_preflight.source_root_hash.clone(),
        source_root_path_hash: path_hash(source_snapshot_root),
        snapshot_copy_root_ref: snapshot_copy_root.display().to_string(),
        snapshot_copy_root_hash: copy_preflight.source_root_hash.clone(),
        snapshot_copy_root_path_hash: path_hash(snapshot_copy_root),
        temp_db_path_hash: path_hash(temp_db_path),
        temp_export_root_hash: dir_manifest_hash(temp_export_root)?,
        temp_export_root_path_hash: path_hash(temp_export_root),
        report_path_hash: path_hash(report_path),
        copied_file_manifest,
        preflight: SqliteSnapshotPreflightSummary {
            source_status: source_preflight.status.clone(),
            copy_status: copy_preflight.status.clone(),
            source_report_hash,
            copy_report_hash: report_hash(&copy_preflight)?,
            source_files_accepted: source_preflight.counts.files_accepted,
            copy_files_accepted: copy_preflight.counts.files_accepted,
            source_blocked_reasons: source_preflight.counts.blocked_reasons,
            copy_blocked_reasons: copy_preflight.counts.blocked_reasons,
        },
        apply_summary,
        export_verification,
        rollback_boundary,
        flags: SqliteSnapshotSafetyFlags::default(),
        failure_point: failure_point.map(|point| format!("{point:?}")),
    };
    write_report(report_path, &report)?;
    Ok(report)
}

fn preflight_config(
    config: &SqliteSnapshotApplyConfig,
    denied_path_markers: &[String],
) -> SqliteProductionPreflightConfig {
    SqliteProductionPreflightConfig {
        primary_workflow_state: PRIMARY_WORKFLOW_STATE.to_string(),
        allowed_sidecars: config.allowed_sidecars.clone(),
        denied_path_markers: denied_path_markers.to_vec(),
    }
}

fn copy_snapshot_files(
    source_snapshot_root: &Path,
    snapshot_copy_root: &Path,
    config: &SqliteSnapshotApplyConfig,
    failure_point: Option<SqliteSnapshotApplyFailurePoint>,
) -> Result<Vec<SqliteCopiedSnapshotFile>, String> {
    let mut names = vec![PRIMARY_WORKFLOW_STATE.to_string()];
    names.extend(config.allowed_sidecars.iter().cloned());
    names.sort();
    names.dedup();

    let mut copied = Vec::new();
    for name in names {
        if name.contains('/') || name.contains('\\') {
            return Err(format!(
                "snapshot_apply_blocked:sidecar_name_must_be_flat:{name}"
            ));
        }
        let source = source_snapshot_root.join(&name);
        if !source.exists() {
            continue;
        }
        let destination = snapshot_copy_root.join(&name);
        let bytes = fs::read(&source)
            .map_err(|error| format!("snapshot_copy_read_failed:{}:{error}", source.display()))?;
        fs::write(&destination, &bytes).map_err(|error| {
            format!(
                "snapshot_copy_write_failed:{}:{error}",
                destination.display()
            )
        })?;
        copied.push(copied_file_manifest_item(&name, &bytes)?);
        if failure_point == Some(SqliteSnapshotApplyFailurePoint::CopyInterruptedBeforeManifest) {
            return Err("injected_failure_copy_interrupted_before_manifest".to_string());
        }
    }
    if !copied
        .iter()
        .any(|file| file.path_ref == PRIMARY_WORKFLOW_STATE)
    {
        return Err("snapshot_apply_blocked:primary_workflow_state_not_copied".to_string());
    }
    Ok(copied)
}

fn copied_file_manifest_item(
    path_ref: &str,
    bytes: &[u8],
) -> Result<SqliteCopiedSnapshotFile, String> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("snapshot_copy_manifest_parse_failed:{path_ref}:{error}"))?;
    Ok(SqliteCopiedSnapshotFile {
        path_ref: path_ref.to_string(),
        path_hash: canonical_json_hash(&json!({ "path_ref": path_ref })),
        file_hash: sha256_hex(bytes),
        size_bytes: bytes.len() as u64,
        schema_version: value
            .get("schema_version")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        revision: value.get("revision").and_then(Value::as_i64),
    })
}

fn write_copy_manifest(
    snapshot_copy_root: &Path,
    manifest: &[SqliteCopiedSnapshotFile],
) -> Result<(), String> {
    write_json_file(
        &snapshot_copy_root.join(COPY_MANIFEST_NAME),
        &json!({
            "schema_version": SNAPSHOT_APPLY_SCHEMA_VERSION,
            "status": "completed",
            "mode": MODE,
            "level": LEVEL_A,
            "copied_files": manifest,
            "manifest_hash": canonical_json_hash(&json!({ "copied_files": manifest }))
        }),
    )
}

fn apply_summary(
    report: &crate::workbench_sqlite_apply::SqliteApplyImportReport,
    db_path: &Path,
) -> Result<SqliteSnapshotApplySummary, String> {
    Ok(SqliteSnapshotApplySummary {
        status: report.status.clone(),
        batch_id: report.batch_id.clone(),
        source_root_hash: report.source_root_hash.clone(),
        records_inserted: report.records_inserted,
        records_skipped: report.records_skipped,
        sources_inserted: report.sources_inserted,
        db_row_counts: db_row_counts(db_path)?,
        runtime_log_alias_policy: report.runtime_log_alias_policy.clone(),
    })
}

fn export_verification(
    db_path: &Path,
    temp_export_root: &Path,
) -> Result<SqliteSnapshotExportVerification, String> {
    verify_db_integrity(db_path)?;
    let manifest =
        export_temp_db_to_json_dry_run(db_path, &temp_export_root.display().to_string())?;
    verify_runtime_log_alias_policy(&manifest.projected_files)?;
    let projected_files = snapshot_projected_files(&manifest.projected_files);
    let projection_hash = projection_hash(&projected_files);
    Ok(SqliteSnapshotExportVerification {
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

fn verify_export_hashes(verification: &SqliteSnapshotExportVerification) -> Result<(), String> {
    if verification.db_export_hash == "injected_export_hash_mismatch" {
        return Err("snapshot_apply_blocked:export_hash_mismatch".to_string());
    }
    if verification
        .projected_files
        .iter()
        .any(|file| file.path == "runtime-log.v1.json")
    {
        return Err("snapshot_apply_blocked:legacy_runtime_log_alias_exported".to_string());
    }
    Ok(())
}

fn write_export_projection(
    db_path: &Path,
    temp_export_root: &Path,
    expected_files: &[SqliteSnapshotProjectedFile],
) -> Result<(), String> {
    let manifest =
        export_temp_db_to_json_dry_run(db_path, &temp_export_root.display().to_string())?;
    reset_dir(temp_export_root)?;
    remove_file_if_exists(&temp_export_root.join("runtime-log.v1.json"))?;
    for file in manifest.projected_files {
        if file.path == "runtime-log.v1.json" {
            return Err("snapshot_apply_blocked:legacy_runtime_log_alias_exported".to_string());
        }
        if file.path.contains('/') || file.path.contains('\\') {
            return Err(format!(
                "snapshot_apply_blocked:projection_file_name_must_be_flat:{}",
                file.path
            ));
        }
        write_json_file(&temp_export_root.join(&file.path), &file.projection)?;
    }
    verify_projection_files(temp_export_root, expected_files)
}

fn verify_projection_files(
    temp_export_root: &Path,
    expected_files: &[SqliteSnapshotProjectedFile],
) -> Result<(), String> {
    for file in expected_files {
        let path = temp_export_root.join(&file.path);
        let bytes = fs::read(&path).map_err(|error| {
            format!(
                "snapshot_apply_blocked:projection_file_missing:{}:{error}",
                file.path
            )
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "snapshot_apply_blocked:projection_file_corrupt:{}:{error}",
                file.path
            )
        })?;
        let actual = canonical_json_hash(&value);
        if actual != file.projected_hash {
            return Err(format!(
                "snapshot_apply_blocked:projection_file_hash_mismatch:{}",
                file.path
            ));
        }
    }
    if temp_export_root.join("runtime-log.v1.json").exists() {
        return Err("snapshot_apply_blocked:legacy_runtime_log_alias_file_present".to_string());
    }
    Ok(())
}

fn rollback_boundary(
    temp_export_root: &Path,
    temp_db_path: &Path,
) -> Result<SqliteSnapshotRollbackBoundary, String> {
    let payload = json!({
        "mode": MODE,
        "level": LEVEL_A,
        "temp_export_root_hash": dir_manifest_hash(temp_export_root)?,
        "temp_db_path_hash": path_hash(temp_db_path),
        "would_disable_db_read_cut": true,
        "would_use_snapshot_projection": true,
        "would_preserve_temp_db_for_audit": true,
        "would_require_supervisor_decision": true,
        "production_restore_performed": false
    });
    Ok(SqliteSnapshotRollbackBoundary {
        status: "rollback_boundary_dry_run_only".to_string(),
        rollback_manifest_hash: canonical_json_hash(&payload),
        rollback_manifest_path_hash: path_hash(&rollback_manifest_path(temp_export_root)),
        would_disable_db_read_cut: true,
        would_use_snapshot_projection: true,
        would_preserve_temp_db_for_audit: true,
        would_require_supervisor_decision: true,
        production_restore_performed: false,
        instructions: vec![
            "would disable DB read-cut before any production recovery".to_string(),
            "would use copied snapshot and exported JSON projection as dry-run recovery inputs"
                .to_string(),
            "would preserve the temporary SQLite DB for audit".to_string(),
            "would require supervisor decision before production restore".to_string(),
            "production restore is not performed by this rehearsal".to_string(),
        ],
    })
}

fn write_rollback_manifest(
    temp_export_root: &Path,
    boundary: &SqliteSnapshotRollbackBoundary,
) -> Result<(), String> {
    write_json_file(
        &rollback_manifest_path(temp_export_root),
        &json!({
            "schema_version": SNAPSHOT_APPLY_SCHEMA_VERSION,
            "status": "completed",
            "mode": MODE,
            "level": LEVEL_A,
            "rollback_boundary": boundary,
            "production_restore_performed": false
        }),
    )
}

fn verify_rollback_manifest(
    temp_export_root: &Path,
) -> Result<SqliteSnapshotRollbackBoundary, String> {
    let path = rollback_manifest_path(temp_export_root);
    let bytes = fs::read(&path)
        .map_err(|error| format!("snapshot_apply_blocked:rollback_manifest_missing:{error}"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("snapshot_apply_blocked:rollback_manifest_corrupt:{error}"))?;
    if value.get("status").and_then(Value::as_str) != Some("completed") {
        remove_file_if_exists(&path)?;
        return Err("snapshot_apply_blocked:rollback_manifest_incomplete".to_string());
    }
    let boundary: SqliteSnapshotRollbackBoundary = serde_json::from_value(
        value
            .get("rollback_boundary")
            .cloned()
            .ok_or_else(|| "snapshot_apply_blocked:rollback_boundary_missing".to_string())?,
    )
    .map_err(|error| format!("snapshot_apply_blocked:rollback_boundary_parse_failed:{error}"))?;
    if boundary.production_restore_performed {
        return Err("snapshot_apply_blocked:rollback_boundary_not_dry_run".to_string());
    }
    Ok(boundary)
}

fn snapshot_projected_files(files: &[SqliteProjectedFile]) -> Vec<SqliteSnapshotProjectedFile> {
    files
        .iter()
        .map(|file| SqliteSnapshotProjectedFile {
            path: file.path.clone(),
            projected_hash: file.projected_hash.clone(),
            record_count: file.record_count,
            redaction_status: "forbidden_sensitive_fields_omitted".to_string(),
        })
        .collect()
}

fn projection_hash(files: &[SqliteSnapshotProjectedFile]) -> String {
    canonical_json_hash(&json!({ "projected_files": files }))
}

fn verify_runtime_log_alias_policy(files: &[SqliteProjectedFile]) -> Result<(), String> {
    if files.iter().any(|file| file.path == "runtime-log.v1.json") {
        return Err("snapshot_apply_blocked:legacy_runtime_log_alias_exported".to_string());
    }
    if !files.iter().any(|file| file.path == "runtime-logs.v1.json") {
        return Err("snapshot_apply_blocked:canonical_runtime_logs_projection_missing".to_string());
    }
    Ok(())
}

fn ensure_preflight_ready(
    label: &str,
    report: &SqliteProductionPreflightReport,
) -> Result<(), String> {
    if report.status != "preflight_ready" || report.counts.blocked_reasons > 0 {
        return Err(format!(
            "snapshot_apply_blocked:{label}_preflight_not_ready:status={}:blocked={}",
            report.status, report.counts.blocked_reasons
        ));
    }
    if report.production_db_created
        || report.production_root_written
        || report.read_cut_enabled
        || report.stop_write_json
        || report.codex_home_touched
    {
        return Err(format!(
            "snapshot_apply_blocked:{label}_preflight_flags_not_false"
        ));
    }
    Ok(())
}

fn validate_level_a_source_snapshot_root(
    path: &Path,
    denied_path_markers: &[String],
) -> Result<(), String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!(
            "snapshot_apply_source_root_required: {}",
            path.display()
        ));
    }
    if denied_path_hit(path, denied_path_markers) {
        return Err(format!(
            "snapshot_apply_source_root_denied: {}",
            path.display()
        ));
    }
    let fixture_root = manifest_r3_a8_fixture_root();
    if path.starts_with(fixture_root) || path.starts_with(std::env::temp_dir()) {
        Ok(())
    } else {
        Err(format!(
            "snapshot_apply_level_a_fixture_source_required: refusing source outside temp or R3-A8 fixtures: {}",
            path.display()
        ))
    }
}

fn validate_temp_roots(
    source_snapshot_root: &Path,
    snapshot_copy_root: &Path,
    temp_db_path: &Path,
    temp_export_root: &Path,
    report_path: &Path,
    denied_path_markers: &[String],
) -> Result<(), String> {
    for path in [
        snapshot_copy_root,
        temp_db_path,
        temp_export_root,
        report_path,
    ] {
        if !path.is_absolute() {
            return Err(format!(
                "snapshot_apply_absolute_path_required:{}",
                path.display()
            ));
        }
        if path.starts_with(source_snapshot_root) {
            return Err(format!(
                "snapshot_apply_blocked:path_inside_source_root_denied:{}",
                path.display()
            ));
        }
        if denied_path_hit(path, denied_path_markers) {
            return Err(format!(
                "snapshot_apply_blocked:denied_path_marker:{}",
                path.display()
            ));
        }
    }
    if !snapshot_copy_root.starts_with(std::env::temp_dir())
        || !temp_db_path.starts_with(std::env::temp_dir())
        || !temp_export_root.starts_with(std::env::temp_dir())
        || !report_path.starts_with(std::env::temp_dir())
    {
        return Err(
            "snapshot_apply_temp_paths_required: copy/db/export/report must stay under temp"
                .to_string(),
        );
    }
    if report_path.starts_with(snapshot_copy_root) || report_path.starts_with(temp_export_root) {
        return Err(format!(
            "snapshot_apply_report_path_must_be_separate_from_copy_and_export:{}",
            report_path.display()
        ));
    }
    Ok(())
}

fn effective_denied_path_markers(config: &SqliteSnapshotApplyConfig) -> Vec<String> {
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

fn verify_db_integrity(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Err("snapshot_apply_blocked:db_unavailable:missing_db_path".to_string());
    }
    let connection =
        Connection::open(db_path).map_err(|error| format!("snapshot_apply_blocked:{error}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("snapshot_apply_blocked:db_integrity_check_failed:{error}"))?;
    if integrity != "ok" {
        return Err(format!(
            "snapshot_apply_blocked:db_integrity_check_failed:{integrity}"
        ));
    }
    let schema_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("snapshot_apply_blocked:db_schema_mismatch:{error}"))?;
    if schema_count != 1 {
        return Err(
            "snapshot_apply_blocked:db_schema_mismatch:missing_schema_migrations".to_string(),
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
            .map_err(|error| format!("snapshot_dir_hash_read_failed:{name}:{error}"))?;
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

fn rollback_manifest_path(temp_export_root: &Path) -> PathBuf {
    temp_export_root.join(ROLLBACK_MANIFEST_NAME)
}

fn write_report(path: &Path, report: &SqliteSnapshotApplyReport) -> Result<(), String> {
    if report.status != "completed" {
        return Err(format!(
            "snapshot_apply_report_status_not_committable:{}",
            report.status
        ));
    }
    if report.flags != SqliteSnapshotSafetyFlags::default() {
        return Err("snapshot_apply_report_flags_not_false".to_string());
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize snapshot apply report failed: {error}"))?,
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

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn manifest_r3_a8_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_snapshot_level_a_copies_applies_exports_and_writes_report() {
        let source = fixture_dir("snapshot-valid-core-chain");
        let paths = prepare_paths("success");

        let report = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            None,
        )
        .expect("snapshot apply success");

        assert_eq!(report.mode, MODE);
        assert_eq!(report.level, LEVEL_A);
        assert_eq!(report.status, "completed");
        assert_eq!(report.flags, SqliteSnapshotSafetyFlags::default());
        assert_eq!(report.source_root_hash, report.snapshot_copy_root_hash);
        assert!(paths.copy_root.join(PRIMARY_WORKFLOW_STATE).exists());
        assert!(paths.copy_root.join(COPY_MANIFEST_NAME).exists());
        assert!(paths.export_root.join("runtime-logs.v1.json").exists());
        assert!(!paths.export_root.join("runtime-log.v1.json").exists());
        assert!(paths.report_path.exists());
        assert!(report.rollback_boundary.would_require_supervisor_decision);
        assert!(!report.rollback_boundary.production_restore_performed);
    }

    #[test]
    fn sqlite_snapshot_level_a_idempotent_rerun_keeps_report_text_stable() {
        let source = fixture_dir("snapshot-idempotent-rerun");
        let paths = prepare_paths("idempotent");
        let config = SqliteSnapshotApplyConfig::default();

        rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &config,
            None,
        )
        .expect("first snapshot apply");
        let first_report = fs::read_to_string(&paths.report_path).expect("first report");
        rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &config,
            None,
        )
        .expect("second snapshot apply");
        let second_report = fs::read_to_string(&paths.report_path).expect("second report");

        assert_eq!(first_report, second_report);
    }

    #[test]
    fn sqlite_snapshot_source_preflight_blocked_by_denied_file_before_copy() {
        let source = fixture_dir("snapshot-preflight-blocked-denied-path");
        let paths = prepare_paths("preflight-blocked");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            None,
        )
        .expect_err("source preflight must block");

        assert!(err.contains("source_preflight_not_ready"));
        assert!(!paths.copy_root.exists());
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_snapshot_rejects_copy_destination_inside_source_root() {
        let source = fixture_dir("snapshot-valid-core-chain");
        let paths = prepare_paths("copy-inside-source");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &source.join("nested-copy"),
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            None,
        )
        .expect_err("copy inside source must reject");

        assert!(err.contains("path_inside_source_root_denied"));
    }

    #[test]
    fn sqlite_snapshot_rejects_report_path_inside_source_root() {
        let source = fixture_dir("snapshot-valid-core-chain");
        let paths = prepare_paths("report-inside-source");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &source.join("report.json"),
            &SqliteSnapshotApplyConfig::default(),
            None,
        )
        .expect_err("report inside source must reject");

        assert!(err.contains("path_inside_source_root_denied"));
    }

    #[test]
    fn sqlite_snapshot_rejects_temp_db_path_inside_source_root() {
        let source = fixture_dir("snapshot-valid-core-chain");
        let paths = prepare_paths("db-inside-source");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &source.join("temp.sqlite"),
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            None,
        )
        .expect_err("db inside source must reject");

        assert!(err.contains("path_inside_source_root_denied"));
    }

    #[test]
    fn sqlite_snapshot_copy_interrupted_before_manifest_writes_no_report() {
        let source = fixture_dir("snapshot-valid-core-chain");
        let paths = prepare_paths("copy-interrupted");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            Some(SqliteSnapshotApplyFailurePoint::CopyInterruptedBeforeManifest),
        )
        .expect_err("copy interruption must stop before report");

        assert!(err.contains("copy_interrupted_before_manifest"));
        assert!(!paths.copy_root.join(COPY_MANIFEST_NAME).exists());
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_snapshot_apply_rejected_corrupt_snapshot_blocks_report() {
        let source = fixture_dir("snapshot-apply-corrupt-blocked");
        let paths = prepare_paths("apply-corrupt");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            Some(SqliteSnapshotApplyFailurePoint::ApplyRejectedCorruptSnapshot),
        )
        .expect_err("corrupt copied snapshot must block apply");

        assert!(err.contains("dry_run_batch_not_applyable") || err.contains("rejected_corrupt"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_snapshot_export_hash_mismatch_blocks_report() {
        let source = fixture_dir("snapshot-export-hash-mismatch-blocked");
        let paths = prepare_paths("export-hash-mismatch");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            Some(SqliteSnapshotApplyFailurePoint::ExportHashMismatch),
        )
        .expect_err("export mismatch must block");

        assert!(err.contains("export_hash_mismatch"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_snapshot_missing_rollback_manifest_blocks_report() {
        let source = fixture_dir("snapshot-rollback-manifest-missing-blocked");
        let paths = prepare_paths("rollback-missing");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            Some(SqliteSnapshotApplyFailurePoint::RollbackManifestMissing),
        )
        .expect_err("missing rollback manifest must block");

        assert!(err.contains("rollback_manifest_missing"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_snapshot_incomplete_rollback_manifest_blocks_report() {
        let source = fixture_dir("snapshot-rollback-manifest-incomplete-blocked");
        let paths = prepare_paths("rollback-incomplete");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            Some(SqliteSnapshotApplyFailurePoint::RollbackManifestIncomplete),
        )
        .expect_err("incomplete rollback manifest must block");

        assert!(err.contains("rollback_manifest_incomplete"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_snapshot_cleanup_failure_keeps_only_temp_artifacts_and_source_unchanged() {
        let source = fixture_dir("snapshot-cleanup-failure-boundary");
        let paths = prepare_paths("cleanup-failure");
        let before_hash = dir_manifest_hash(&source).expect("source hash before");

        let report = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &paths.report_path,
            &SqliteSnapshotApplyConfig::default(),
            Some(SqliteSnapshotApplyFailurePoint::CleanupFailureAfterRollbackBoundary),
        )
        .expect("cleanup failure boundary remains completed");

        let after_hash = dir_manifest_hash(&source).expect("source hash after");
        assert_eq!(before_hash, after_hash);
        assert!(paths.copy_root.exists());
        assert!(paths.export_root.exists());
        assert!(paths.db_path.exists());
        assert!(paths.report_path.exists());
        assert!(report
            .rollback_boundary
            .instructions
            .iter()
            .any(|instruction| instruction.contains("temp artifacts preserved")));
    }

    #[test]
    fn sqlite_snapshot_rejects_denied_config_marker_even_when_custom_config_is_empty() {
        let source = fixture_dir("snapshot-valid-core-chain");
        let paths = prepare_paths("denied-report-path");
        let config = SqliteSnapshotApplyConfig {
            denied_path_markers: Vec::new(),
            ..SqliteSnapshotApplyConfig::default()
        };
        let denied_report = std::env::temp_dir()
            .join(".codex-r3-a8-denied")
            .join("report.json");

        let err = rehearse_copied_snapshot_apply_level_a(
            &source,
            &paths.copy_root,
            &paths.db_path,
            &paths.export_root,
            &denied_report,
            &config,
            None,
        )
        .expect_err("default denied markers must remain active");

        assert!(err.contains("denied_path_marker"));
    }

    struct Paths {
        copy_root: PathBuf,
        db_path: PathBuf,
        export_root: PathBuf,
        report_path: PathBuf,
    }

    fn fixture_dir(name: &str) -> PathBuf {
        manifest_r3_a8_fixture_root().join(name)
    }

    fn prepare_paths(label: &str) -> Paths {
        let unique = unique_label(label);
        let root = std::env::temp_dir().join(format!("workbench-r3-a8-{unique}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root");
        Paths {
            copy_root: root.join("copy"),
            db_path: root.join("temp.sqlite"),
            export_root: root.join("export"),
            report_path: root.join("reports").join("snapshot-report.json"),
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
