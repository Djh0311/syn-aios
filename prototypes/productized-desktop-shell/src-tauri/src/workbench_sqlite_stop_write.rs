use crate::utils::fs_ops::remove_file_if_exists;
use crate::utils::hash::sha256_hex_bytes as sha256_hex;
use crate::workbench_sqlite_exporter::{export_temp_db_to_json_dry_run, SqliteProjectedFile};
use crate::workbench_sqlite_importer::canonical_json_hash;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const LEVEL_A: &str = "level_a_fixture";
const MODE: &str = "stop_write_json_decision";
const DECISION_MODE: &str = "level_a_fixture_stop_write_decision";
const SCHEMA_VERSION: &str = "workbench_sqlite_stop_write_decision.v1";
const WORKFLOW_STATE_SUMMARY_READ_MODEL: &str = "workflow_state_summary";
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
pub(crate) enum SqliteStopWriteFailurePoint {
    AfterPreconditionsBeforeReportCommit,
    SourceMutationDetected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SqliteStopWriteLevelBEvidence {
    pub(crate) production_db_apply_level_b_completed: bool,
    pub(crate) limited_read_cut_level_b_completed: bool,
    pub(crate) production_observation_level_b_completed: bool,
}

impl SqliteStopWriteLevelBEvidence {
    fn complete(&self) -> bool {
        self.production_db_apply_level_b_completed
            && self.limited_read_cut_level_b_completed
            && self.production_observation_level_b_completed
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteStopWriteDecisionReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) level: String,
    pub(crate) status: String,
    pub(crate) decision_mode: String,
    pub(crate) supervisor_decision: String,
    pub(crate) read_model_name: String,
    pub(crate) preconditions: Vec<SqliteStopWritePrecondition>,
    pub(crate) db_path_hash: String,
    pub(crate) expected_db_hash: Option<String>,
    pub(crate) actual_db_hash: Option<String>,
    pub(crate) expected_fallback_hash: Option<String>,
    pub(crate) actual_fallback_hash: String,
    pub(crate) expected_projection_hash: Option<String>,
    pub(crate) actual_projection_hash: String,
    pub(crate) expected_observation_report_hash: Option<String>,
    pub(crate) actual_observation_report_hash: Option<String>,
    pub(crate) rollback_manifest_hash: Option<String>,
    pub(crate) db_export_hash: Option<String>,
    pub(crate) projected_file_counts: BTreeMap<String, usize>,
    pub(crate) rollback_drill: SqliteStopWriteRollbackDrill,
    pub(crate) safety_flags: SqliteStopWriteSafetyFlags,
    pub(crate) before_source_hashes: Vec<SqliteStopWriteSourceFileHash>,
    pub(crate) after_source_hashes: Vec<SqliteStopWriteSourceFileHash>,
    pub(crate) failure_point: Option<String>,
    pub(crate) do_not_claim: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteStopWritePrecondition {
    pub(crate) name: String,
    pub(crate) satisfied: bool,
    pub(crate) detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteStopWriteRollbackDrill {
    pub(crate) status: String,
    pub(crate) would_disable_stop_write: bool,
    pub(crate) would_re_enable_json_sidecar_write_path: bool,
    pub(crate) would_use_last_verified_json_projection: bool,
    pub(crate) would_preserve_db_for_audit: bool,
    pub(crate) would_require_supervisor_decision: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) instructions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteStopWriteSafetyFlags {
    pub(crate) stop_write_decision_recorded: bool,
    pub(crate) stop_write_json: bool,
    pub(crate) source_json_written: bool,
    pub(crate) sidecar_written: bool,
    pub(crate) product_global_write_path_changed: bool,
    pub(crate) product_global_read_path_changed: bool,
    pub(crate) app_startup_writes_db: bool,
    pub(crate) tauri_command_writes_db: bool,
    pub(crate) ui_writes_db: bool,
    pub(crate) production_restore_performed: bool,
    pub(crate) codex_home_touched: bool,
}

impl SqliteStopWriteSafetyFlags {
    fn recorded() -> Self {
        Self {
            stop_write_decision_recorded: true,
            stop_write_json: false,
            source_json_written: false,
            sidecar_written: false,
            product_global_write_path_changed: false,
            product_global_read_path_changed: false,
            app_startup_writes_db: false,
            tauri_command_writes_db: false,
            ui_writes_db: false,
            production_restore_performed: false,
            codex_home_touched: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteStopWriteSourceFileHash {
    pub(crate) path_ref: String,
    pub(crate) file_hash: String,
    pub(crate) size_bytes: u64,
}

struct StopWriteObservedInputs {
    db_path_hash: String,
    actual_db_hash: Option<String>,
    actual_fallback_hash: String,
    actual_projection_hash: String,
    actual_observation_report_hash: Option<String>,
    rollback_manifest_hash: Option<String>,
    db_export_hash: Option<String>,
    projected_file_counts: BTreeMap<String, usize>,
    before_source_hashes: Vec<SqliteStopWriteSourceFileHash>,
    after_source_hashes: Vec<SqliteStopWriteSourceFileHash>,
}

pub(crate) fn rehearse_stop_write_decision_level_a(
    decision_mode: &str,
    decision_actor: &str,
    supervisor_decision: Option<&str>,
    read_model_name: &str,
    db_path: &Path,
    json_fallback_root: &Path,
    last_verified_projection_root: &Path,
    stop_write_report_path: &Path,
    rollback_manifest_path: &Path,
    observation_report_path: &Path,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    expected_projection_hash: Option<&str>,
    expected_observation_report_hash: Option<&str>,
    allowed_read_models: &BTreeSet<String>,
    denied_path_markers: &[String],
    level_b_evidence: &SqliteStopWriteLevelBEvidence,
    failure_point: Option<SqliteStopWriteFailurePoint>,
) -> Result<SqliteStopWriteDecisionReport, String> {
    validate_decision_mode(decision_mode)?;
    validate_decision_actor(decision_actor)?;
    validate_read_model(read_model_name, allowed_read_models)?;
    let supervisor_decision = supervisor_decision
        .ok_or_else(|| "stop_write_blocked:missing_supervisor_decision".to_string())?;
    validate_supervisor_decision(supervisor_decision)?;
    let denied_markers = effective_denied_markers(denied_path_markers);
    validate_paths(
        db_path,
        json_fallback_root,
        last_verified_projection_root,
        stop_write_report_path,
        rollback_manifest_path,
        observation_report_path,
        &denied_markers,
    )?;
    remove_file_if_exists(stop_write_report_path)?;

    let observed = observed_inputs(
        db_path,
        json_fallback_root,
        last_verified_projection_root,
        rollback_manifest_path,
        observation_report_path,
        failure_point,
    )?;
    let preconditions = preconditions(
        &observed,
        expected_db_hash,
        expected_fallback_hash,
        expected_projection_hash,
        expected_observation_report_hash,
        level_b_evidence,
    );

    let status = match supervisor_decision {
        "prepare_only" => "not_ready",
        "reject_stop_write" => "rejected_by_supervisor",
        "approve_stop_write" => {
            let failed = preconditions
                .iter()
                .filter(|condition| !condition.satisfied)
                .map(|condition| condition.name.clone())
                .collect::<Vec<_>>();
            if !failed.is_empty() {
                remove_file_if_exists(stop_write_report_path)?;
                return Err(format!(
                    "stop_write_blocked:preconditions_not_met:{}",
                    failed.join(",")
                ));
            }
            if failure_point
                == Some(SqliteStopWriteFailurePoint::AfterPreconditionsBeforeReportCommit)
            {
                remove_file_if_exists(stop_write_report_path)?;
                return Err("injected_failure_after_preconditions_before_report_commit".to_string());
            }
            "ready_but_not_executed"
        }
        _ => unreachable!("supervisor decision already validated"),
    };

    let report = SqliteStopWriteDecisionReport {
        schema_version: SCHEMA_VERSION.to_string(),
        mode: MODE.to_string(),
        level: LEVEL_A.to_string(),
        status: status.to_string(),
        decision_mode: decision_mode.to_string(),
        supervisor_decision: supervisor_decision.to_string(),
        read_model_name: read_model_name.to_string(),
        preconditions,
        db_path_hash: observed.db_path_hash,
        expected_db_hash: expected_db_hash.map(ToString::to_string),
        actual_db_hash: observed.actual_db_hash,
        expected_fallback_hash: expected_fallback_hash.map(ToString::to_string),
        actual_fallback_hash: observed.actual_fallback_hash,
        expected_projection_hash: expected_projection_hash.map(ToString::to_string),
        actual_projection_hash: observed.actual_projection_hash,
        expected_observation_report_hash: expected_observation_report_hash.map(ToString::to_string),
        actual_observation_report_hash: observed.actual_observation_report_hash,
        rollback_manifest_hash: observed.rollback_manifest_hash,
        db_export_hash: observed.db_export_hash,
        projected_file_counts: observed.projected_file_counts,
        rollback_drill: rollback_drill(),
        safety_flags: SqliteStopWriteSafetyFlags::recorded(),
        before_source_hashes: observed.before_source_hashes,
        after_source_hashes: observed.after_source_hashes,
        failure_point: failure_point.map(|point| format!("{point:?}")),
        do_not_claim: do_not_claim(),
    };
    write_report(stop_write_report_path, &report)?;
    Ok(report)
}

fn validate_decision_mode(decision_mode: &str) -> Result<(), String> {
    if decision_mode == DECISION_MODE {
        Ok(())
    } else {
        Err(format!(
            "stop_write_blocked:unsupported_decision_mode:{decision_mode}"
        ))
    }
}

fn validate_supervisor_decision(decision: &str) -> Result<(), String> {
    match decision {
        "prepare_only" | "reject_stop_write" | "approve_stop_write" => Ok(()),
        _ => Err(format!(
            "stop_write_blocked:invalid_supervisor_decision:{decision}"
        )),
    }
}

fn validate_decision_actor(actor: &str) -> Result<(), String> {
    match actor {
        "global_supervisor" | "supervisor_user" => Ok(()),
        _ => Err(format!("stop_write_blocked:invalid_decision_actor:{actor}")),
    }
}

fn validate_read_model(
    read_model_name: &str,
    allowed_read_models: &BTreeSet<String>,
) -> Result<(), String> {
    if read_model_name != WORKFLOW_STATE_SUMMARY_READ_MODEL {
        return Err(format!(
            "stop_write_blocked:unsupported_read_model:{read_model_name}"
        ));
    }
    if !allowed_read_models.contains(read_model_name) {
        return Err(format!(
            "stop_write_blocked:read_model_not_allowed:{read_model_name}"
        ));
    }
    Ok(())
}

fn validate_paths(
    db_path: &Path,
    json_fallback_root: &Path,
    last_verified_projection_root: &Path,
    stop_write_report_path: &Path,
    rollback_manifest_path: &Path,
    observation_report_path: &Path,
    denied_path_markers: &[String],
) -> Result<(), String> {
    for path in [
        db_path,
        json_fallback_root,
        last_verified_projection_root,
        stop_write_report_path,
        rollback_manifest_path,
        observation_report_path,
    ] {
        if !path.is_absolute() {
            return Err(format!(
                "stop_write_blocked:absolute_path_required:{}",
                path.display()
            ));
        }
        if denied_path_hit(path, denied_path_markers) {
            return Err(format!(
                "stop_write_blocked:denied_path_marker:{}",
                path.display()
            ));
        }
        if !is_allowed_level_a_path(path) {
            return Err(format!(
                "stop_write_blocked:level_a_temp_or_fixture_path_required:{}",
                path.display()
            ));
        }
    }
    if stop_write_report_path.starts_with(json_fallback_root)
        || rollback_manifest_path.starts_with(json_fallback_root)
        || observation_report_path.starts_with(json_fallback_root)
    {
        return Err("stop_write_blocked:report_or_manifest_inside_source_root_denied".to_string());
    }
    Ok(())
}

fn observed_inputs(
    db_path: &Path,
    json_fallback_root: &Path,
    last_verified_projection_root: &Path,
    rollback_manifest_path: &Path,
    observation_report_path: &Path,
    failure_point: Option<SqliteStopWriteFailurePoint>,
) -> Result<StopWriteObservedInputs, String> {
    let before_source_hashes = source_file_hashes(json_fallback_root)?;
    let mut after_source_hashes = before_source_hashes.clone();
    if failure_point == Some(SqliteStopWriteFailurePoint::SourceMutationDetected) {
        after_source_hashes.push(SqliteStopWriteSourceFileHash {
            path_ref: "injected-source-mutation".to_string(),
            file_hash: "injected".to_string(),
            size_bytes: 0,
        });
    }
    let actual_db_hash = if db_path.exists() {
        Some(file_hash(db_path)?)
    } else {
        None
    };
    let export_manifest = if db_path.exists() {
        Some(export_temp_db_to_json_dry_run(
            db_path,
            &last_verified_projection_root.display().to_string(),
        )?)
    } else {
        None
    };
    let db_export_hash = export_manifest
        .as_ref()
        .map(|manifest| manifest.export_hash.clone());
    let projected_file_counts = export_manifest
        .as_ref()
        .map(projected_file_counts)
        .unwrap_or_default();
    Ok(StopWriteObservedInputs {
        db_path_hash: path_hash(db_path),
        actual_db_hash,
        actual_fallback_hash: root_manifest_hash(json_fallback_root)?,
        actual_projection_hash: root_manifest_hash(last_verified_projection_root)?,
        actual_observation_report_hash: optional_file_hash(observation_report_path)?,
        rollback_manifest_hash: verify_rollback_manifest(rollback_manifest_path)?,
        db_export_hash,
        projected_file_counts,
        before_source_hashes,
        after_source_hashes,
    })
}

fn preconditions(
    observed: &StopWriteObservedInputs,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    expected_projection_hash: Option<&str>,
    expected_observation_report_hash: Option<&str>,
    level_b_evidence: &SqliteStopWriteLevelBEvidence,
) -> Vec<SqliteStopWritePrecondition> {
    vec![
        condition(
            "level_b_evidence_complete",
            level_b_evidence.complete(),
            "A9/A10/A11 Level B or equivalent production evidence must be complete",
        ),
        condition(
            "db_exists",
            observed.actual_db_hash.is_some(),
            "production DB path must exist",
        ),
        condition(
            "db_hash_matches",
            expected_db_hash
                .zip(observed.actual_db_hash.as_deref())
                .map_or(false, |(expected, actual)| expected == actual),
            "expected DB hash must match actual DB hash",
        ),
        condition(
            "fallback_hash_matches",
            expected_fallback_hash
                .map_or(false, |expected| expected == observed.actual_fallback_hash),
            "verified JSON fallback hash must match",
        ),
        condition(
            "projection_hash_matches",
            expected_projection_hash.map_or(false, |expected| {
                expected == observed.actual_projection_hash
            }),
            "last verified projection hash must match",
        ),
        condition(
            "observation_report_hash_matches",
            expected_observation_report_hash
                .zip(observed.actual_observation_report_hash.as_deref())
                .map_or(false, |(expected, actual)| expected == actual),
            "observation report hash must match",
        ),
        condition(
            "rollback_manifest_complete",
            observed.rollback_manifest_hash.is_some(),
            "rollback manifest must be complete and dry-run only",
        ),
        condition(
            "source_hashes_unchanged",
            observed.before_source_hashes == observed.after_source_hashes,
            "source JSON / sidecar hashes must not change",
        ),
        condition(
            "no_product_runtime_path_changed",
            true,
            "no startup, Tauri command, UI, product global read/write path is changed",
        ),
        condition(
            "production_restore_not_performed",
            true,
            "production restore is not performed by Level A",
        ),
    ]
}

fn condition(name: &str, satisfied: bool, detail: &str) -> SqliteStopWritePrecondition {
    SqliteStopWritePrecondition {
        name: name.to_string(),
        satisfied,
        detail: detail.to_string(),
    }
}

fn rollback_drill() -> SqliteStopWriteRollbackDrill {
    SqliteStopWriteRollbackDrill {
        status: "rollback_drill_only".to_string(),
        would_disable_stop_write: true,
        would_re_enable_json_sidecar_write_path: true,
        would_use_last_verified_json_projection: true,
        would_preserve_db_for_audit: true,
        would_require_supervisor_decision: true,
        production_restore_performed: false,
        instructions: vec![
            "would disable stop-write mode before recovery".to_string(),
            "would re-enable JSON / sidecar write path".to_string(),
            "would use last verified JSON projection".to_string(),
            "would preserve DB for audit".to_string(),
            "would require supervisor decision before production restore".to_string(),
            "production restore is not performed by this rehearsal".to_string(),
        ],
    }
}

fn write_report(path: &Path, report: &SqliteStopWriteDecisionReport) -> Result<(), String> {
    if !matches!(
        report.status.as_str(),
        "not_ready" | "rejected_by_supervisor" | "ready_but_not_executed"
    ) {
        return Err(format!(
            "stop_write_report_status_not_committable:{}",
            report.status
        ));
    }
    if report.safety_flags.stop_write_json
        || report.safety_flags.source_json_written
        || report.safety_flags.sidecar_written
        || report.safety_flags.product_global_write_path_changed
        || report.safety_flags.product_global_read_path_changed
        || report.safety_flags.app_startup_writes_db
        || report.safety_flags.tauri_command_writes_db
        || report.safety_flags.ui_writes_db
        || report.safety_flags.production_restore_performed
        || report.safety_flags.codex_home_touched
    {
        return Err("stop_write_report_forbidden_flag_true".to_string());
    }
    write_json_file(
        path,
        &serde_json::to_value(report)
            .map_err(|error| format!("serialize stop-write report failed: {error}"))?,
    )
}

fn verify_rollback_manifest(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("stop_write_blocked:rollback_manifest_read_failed:{error}"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("stop_write_blocked:rollback_manifest_corrupt:{error}"))?;
    if value.get("status").and_then(Value::as_str) != Some("completed") {
        return Ok(None);
    }
    let restore_performed = value
        .get("rollback_boundary")
        .and_then(|boundary| boundary.get("production_restore_performed"))
        .or_else(|| value.get("production_restore_performed"))
        .and_then(Value::as_bool);
    if restore_performed != Some(false) {
        return Ok(None);
    }
    Ok(Some(file_hash(path)?))
}

fn source_file_hashes(root: &Path) -> Result<Vec<SqliteStopWriteSourceFileHash>, String> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut hashes = Vec::new();
    for entry in sorted_entries(root)? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let bytes = fs::read(&path)
            .map_err(|error| format!("stop_write_source_hash_read_failed:{name}:{error}"))?;
        hashes.push(SqliteStopWriteSourceFileHash {
            path_ref: name,
            file_hash: sha256_hex(&bytes),
            size_bytes: bytes.len() as u64,
        });
    }
    Ok(hashes)
}

fn root_manifest_hash(root: &Path) -> Result<String, String> {
    if !root.exists() {
        return Ok(canonical_json_hash(&json!({ "entries": [] })));
    }
    let entries = source_file_hashes(root)?
        .into_iter()
        .map(|entry| {
            json!({
                "path": entry.path_ref,
                "hash": entry.file_hash,
                "size_bytes": entry.size_bytes
            })
        })
        .collect::<Vec<_>>();
    Ok(canonical_json_hash(&json!({ "entries": entries })))
}

fn projected_file_counts(
    manifest: &crate::workbench_sqlite_exporter::SqliteExportDryRunManifest,
) -> BTreeMap<String, usize> {
    manifest
        .projected_files
        .iter()
        .map(|file: &SqliteProjectedFile| (file.path.clone(), file.record_count))
        .collect()
}

fn optional_file_hash(path: &Path) -> Result<Option<String>, String> {
    if path.exists() {
        Ok(Some(file_hash(path)?))
    } else {
        Ok(None)
    }
}

fn file_hash(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("read hash failed {}: {error}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

fn path_hash(path: &Path) -> String {
    canonical_json_hash(&json!({ "path": path.display().to_string() }))
}

fn sorted_entries(root: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("read dir failed {}: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read dir entry failed {}: {error}", root.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
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

fn effective_denied_markers(markers: &[String]) -> Vec<String> {
    let mut merged = DEFAULT_DENIED_PATH_MARKERS
        .iter()
        .map(|marker| marker.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    merged.extend(markers.iter().map(|marker| marker.to_ascii_lowercase()));
    merged.into_iter().collect()
}

fn denied_path_hit(path: &Path, markers: &[String]) -> bool {
    let haystack = path.to_string_lossy().to_ascii_lowercase();
    markers
        .iter()
        .any(|marker| haystack.contains(&marker.to_ascii_lowercase()))
}

fn is_allowed_level_a_path(path: &Path) -> bool {
    path.starts_with(std::env::temp_dir()) || path.starts_with(r3_a12_fixture_root())
}

fn r3_a12_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("r3-a12")
}

fn do_not_claim() -> Vec<String> {
    vec![
        "JSON / sidecar stop-write completed".to_string(),
        "production read-cut completed".to_string(),
        "app real SQLite write path enabled".to_string(),
        "production observation Level B completed".to_string(),
        "rollback production workflow completed".to_string(),
        "R3 completed".to_string(),
        "multi-agent parallel real execution unlocked".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbench_sqlite_apply::apply_fixture_dir_to_temp_db;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_stop_write_prepare_only_not_ready_without_stop_write() {
        let paths = prepare_paths("prepare-only");
        let report = rehearse_stop_write_decision_level_a(
            DECISION_MODE,
            "global_supervisor",
            Some("prepare_only"),
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &paths.observation_report_path,
            None,
            None,
            None,
            None,
            &allowed_read_models(),
            &[],
            &incomplete_level_b_evidence(),
            None,
        )
        .expect("prepare only report");

        assert_eq!(report.status, "not_ready");
        assert!(!report.safety_flags.stop_write_json);
        assert!(!report.safety_flags.source_json_written);
        assert_eq!(report.before_source_hashes, report.after_source_hashes);
        assert!(paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_rejected_by_supervisor_records_rejection() {
        let paths = prepare_ready_paths("reject");
        let report = ready_call(&paths, "reject_stop_write", &incomplete_level_b_evidence())
            .expect("reject report");

        assert_eq!(report.status, "rejected_by_supervisor");
        assert!(!report.safety_flags.stop_write_json);
    }

    #[test]
    fn sqlite_stop_write_approve_without_level_b_evidence_blocks() {
        let paths = prepare_ready_paths("approve-without-level-b");
        let err = ready_call(&paths, "approve_stop_write", &incomplete_level_b_evidence())
            .expect_err("missing level b evidence blocks");

        assert!(err.contains("level_b_evidence_complete"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_ready_but_not_executed_with_fixture_evidence() {
        let paths = prepare_ready_paths("ready");
        let report = ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect("ready but not executed");

        assert_eq!(report.status, "ready_but_not_executed");
        assert!(report.safety_flags.stop_write_decision_recorded);
        assert!(!report.safety_flags.stop_write_json);
        assert!(
            report
                .rollback_drill
                .would_re_enable_json_sidecar_write_path
        );
        assert!(!report.rollback_drill.production_restore_performed);
    }

    #[test]
    fn sqlite_stop_write_db_missing_blocks() {
        let mut paths = prepare_ready_paths("db-missing");
        remove_file_if_exists(&paths.db_path).expect("remove db");
        paths.expected_db_hash = None;
        let err = ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("missing db blocks");

        assert!(err.contains("db_exists"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_db_hash_mismatch_blocks() {
        let mut paths = prepare_ready_paths("db-hash-mismatch");
        paths.expected_db_hash = Some("wrong".to_string());
        let err = ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("db hash mismatch blocks");

        assert!(err.contains("db_hash_matches"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_fallback_hash_mismatch_blocks() {
        let mut paths = prepare_ready_paths("fallback-hash-mismatch");
        paths.expected_fallback_hash = Some("wrong".to_string());
        let err = ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("fallback hash mismatch blocks");

        assert!(err.contains("fallback_hash_matches"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_projection_hash_mismatch_blocks() {
        let mut paths = prepare_ready_paths("projection-hash-mismatch");
        paths.expected_projection_hash = Some("wrong".to_string());
        let err = ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("projection hash mismatch blocks");

        assert!(err.contains("projection_hash_matches"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_observation_report_missing_or_mismatch_blocks() {
        let mut missing = prepare_ready_paths("observation-missing");
        remove_file_if_exists(&missing.observation_report_path).expect("remove observation report");
        missing.expected_observation_report_hash = None;
        let err = ready_call(&missing, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("missing observation report blocks");
        assert!(err.contains("observation_report_hash_matches"));

        let mut mismatch = prepare_ready_paths("observation-mismatch");
        mismatch.expected_observation_report_hash = Some("wrong".to_string());
        let err = ready_call(
            &mismatch,
            "approve_stop_write",
            &complete_level_b_evidence(),
        )
        .expect_err("observation report mismatch blocks");
        assert!(err.contains("observation_report_hash_matches"));
    }

    #[test]
    fn sqlite_stop_write_rollback_manifest_missing_or_incomplete_blocks() {
        let missing = prepare_ready_paths("rollback-missing");
        remove_file_if_exists(&missing.rollback_manifest_path).expect("remove rollback manifest");
        let err = ready_call(&missing, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("missing rollback blocks");
        assert!(err.contains("rollback_manifest_complete"));

        let incomplete = prepare_ready_paths("rollback-incomplete");
        write_json_file(
            &incomplete.rollback_manifest_path,
            &json!({
                "schema_version": SCHEMA_VERSION,
                "status": "manifest_commit_failed_before_complete",
                "production_restore_performed": false
            }),
        )
        .expect("write incomplete rollback");
        let err = ready_call(
            &incomplete,
            "approve_stop_write",
            &complete_level_b_evidence(),
        )
        .expect_err("incomplete rollback blocks");
        assert!(err.contains("rollback_manifest_complete"));
    }

    #[test]
    fn sqlite_stop_write_source_mutation_detected_blocks() {
        let paths = prepare_ready_paths("source-mutation");
        let err = ready_call_with_failure(
            &paths,
            "approve_stop_write",
            &complete_level_b_evidence(),
            Some(SqliteStopWriteFailurePoint::SourceMutationDetected),
        )
        .expect_err("source mutation blocks");

        assert!(err.contains("source_hashes_unchanged"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_denied_path_marker_blocks() {
        let paths = prepare_ready_paths("denied-path");
        let err = rehearse_stop_write_decision_level_a(
            DECISION_MODE,
            "global_supervisor",
            Some("prepare_only"),
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &paths.observation_report_path,
            paths.expected_db_hash.as_deref(),
            paths.expected_fallback_hash.as_deref(),
            paths.expected_projection_hash.as_deref(),
            paths.expected_observation_report_hash.as_deref(),
            &allowed_read_models(),
            &[paths
                .fallback_root
                .file_name()
                .expect("fallback name")
                .to_string_lossy()
                .to_string()],
            &complete_level_b_evidence(),
            None,
        )
        .expect_err("denied path blocks");

        assert!(err.contains("denied_path_marker"));
    }

    #[test]
    fn sqlite_stop_write_rollback_restore_performed_blocks() {
        let paths = prepare_ready_paths("restore-performed");
        write_json_file(
            &paths.rollback_manifest_path,
            &json!({
                "schema_version": SCHEMA_VERSION,
                "status": "completed",
                "rollback_boundary": {
                    "production_restore_performed": true
                }
            }),
        )
        .expect("write restore performed rollback");
        let err = ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect_err("restore performed blocks");

        assert!(err.contains("rollback_manifest_complete"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_after_preconditions_before_report_commit_leaves_no_report() {
        let paths = prepare_ready_paths("after-preconditions");
        let err = ready_call_with_failure(
            &paths,
            "approve_stop_write",
            &complete_level_b_evidence(),
            Some(SqliteStopWriteFailurePoint::AfterPreconditionsBeforeReportCommit),
        )
        .expect_err("after preconditions failure");

        assert!(err.contains("after_preconditions_before_report_commit"));
        assert!(!paths.report_path.exists());
    }

    #[test]
    fn sqlite_stop_write_sensitive_redaction_and_idempotent_rerun() {
        let paths = prepare_ready_paths("sensitive-idempotent");
        ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect("first ready");
        let first = fs::read_to_string(&paths.report_path).expect("first report");
        ready_call(&paths, "approve_stop_write", &complete_level_b_evidence())
            .expect("second ready");
        let second = fs::read_to_string(&paths.report_path).expect("second report");

        assert_eq!(first, second);
        assert!(!first.contains("secret_value"));
        assert!(!first.contains("provider credential value"));
        assert!(!first.contains("full transcript body"));
        assert!(!first.contains("prompt body value"));
    }

    #[test]
    fn sqlite_stop_write_missing_supervisor_or_invalid_decision_blocks() {
        let paths = prepare_paths("invalid-decision");
        let err = rehearse_stop_write_decision_level_a(
            DECISION_MODE,
            "global_supervisor",
            None,
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &paths.observation_report_path,
            None,
            None,
            None,
            None,
            &allowed_read_models(),
            &[],
            &incomplete_level_b_evidence(),
            None,
        )
        .expect_err("missing supervisor blocks");
        assert!(err.contains("missing_supervisor_decision"));

        let err = rehearse_stop_write_decision_level_a(
            DECISION_MODE,
            "global_supervisor",
            Some("developer_auto_approve"),
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &paths.observation_report_path,
            None,
            None,
            None,
            None,
            &allowed_read_models(),
            &[],
            &incomplete_level_b_evidence(),
            None,
        )
        .expect_err("invalid supervisor decision blocks");
        assert!(err.contains("invalid_supervisor_decision"));

        let err = rehearse_stop_write_decision_level_a(
            DECISION_MODE,
            "worker",
            Some("approve_stop_write"),
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &paths.observation_report_path,
            None,
            None,
            None,
            None,
            &allowed_read_models(),
            &[],
            &incomplete_level_b_evidence(),
            None,
        )
        .expect_err("non-supervisor actor blocks");
        assert!(err.contains("invalid_decision_actor"));
    }

    struct StopWritePaths {
        db_path: PathBuf,
        fallback_root: PathBuf,
        projection_root: PathBuf,
        report_path: PathBuf,
        rollback_manifest_path: PathBuf,
        observation_report_path: PathBuf,
        expected_db_hash: Option<String>,
        expected_fallback_hash: Option<String>,
        expected_projection_hash: Option<String>,
        expected_observation_report_hash: Option<String>,
    }

    fn ready_call(
        paths: &StopWritePaths,
        decision: &str,
        evidence: &SqliteStopWriteLevelBEvidence,
    ) -> Result<SqliteStopWriteDecisionReport, String> {
        ready_call_with_failure(paths, decision, evidence, None)
    }

    fn ready_call_with_failure(
        paths: &StopWritePaths,
        decision: &str,
        evidence: &SqliteStopWriteLevelBEvidence,
        failure_point: Option<SqliteStopWriteFailurePoint>,
    ) -> Result<SqliteStopWriteDecisionReport, String> {
        rehearse_stop_write_decision_level_a(
            DECISION_MODE,
            "global_supervisor",
            Some(decision),
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.report_path,
            &paths.rollback_manifest_path,
            &paths.observation_report_path,
            paths.expected_db_hash.as_deref(),
            paths.expected_fallback_hash.as_deref(),
            paths.expected_projection_hash.as_deref(),
            paths.expected_observation_report_hash.as_deref(),
            &allowed_read_models(),
            &[],
            evidence,
            failure_point,
        )
    }

    fn prepare_paths(label: &str) -> StopWritePaths {
        let root = temp_root(label);
        let fallback_root = fixture_dir("stop-write-workflow-summary");
        StopWritePaths {
            db_path: root.join("workbench.sqlite"),
            fallback_root,
            projection_root: root.join("projection"),
            report_path: root.join("stop-write-report.json"),
            rollback_manifest_path: root.join("rollback-manifest.json"),
            observation_report_path: root.join("observation-report.json"),
            expected_db_hash: None,
            expected_fallback_hash: None,
            expected_projection_hash: None,
            expected_observation_report_hash: None,
        }
    }

    fn prepare_ready_paths(label: &str) -> StopWritePaths {
        let paths = prepare_paths(label);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None).expect("apply db");
        write_projection_root(&paths.db_path, &paths.projection_root).expect("write projection");
        write_json_file(
            &paths.observation_report_path,
            &json!({
                "schema_version": "workbench_sqlite_production_observation.v1",
                "status": "stable_verified",
                "production_restore_performed": false
            }),
        )
        .expect("write observation report");
        write_json_file(
            &paths.rollback_manifest_path,
            &json!({
                "schema_version": SCHEMA_VERSION,
                "status": "completed",
                "rollback_boundary": {
                    "production_restore_performed": false
                }
            }),
        )
        .expect("write rollback manifest");
        StopWritePaths {
            expected_db_hash: Some(file_hash(&paths.db_path).expect("db hash")),
            expected_fallback_hash: Some(
                root_manifest_hash(&paths.fallback_root).expect("fallback hash"),
            ),
            expected_projection_hash: Some(
                root_manifest_hash(&paths.projection_root).expect("projection hash"),
            ),
            expected_observation_report_hash: Some(
                file_hash(&paths.observation_report_path).expect("observation hash"),
            ),
            ..paths
        }
    }

    fn write_projection_root(db_path: &Path, projection_root: &Path) -> Result<(), String> {
        if projection_root.exists() {
            fs::remove_dir_all(projection_root)
                .map_err(|error| format!("remove projection failed: {error}"))?;
        }
        fs::create_dir_all(projection_root)
            .map_err(|error| format!("create projection failed: {error}"))?;
        let manifest =
            export_temp_db_to_json_dry_run(db_path, &projection_root.display().to_string())?;
        for file in manifest.projected_files {
            write_json_file(&projection_root.join(file.path), &file.projection)?;
        }
        Ok(())
    }

    fn allowed_read_models() -> BTreeSet<String> {
        [WORKFLOW_STATE_SUMMARY_READ_MODEL.to_string()]
            .into_iter()
            .collect()
    }

    fn incomplete_level_b_evidence() -> SqliteStopWriteLevelBEvidence {
        SqliteStopWriteLevelBEvidence {
            production_db_apply_level_b_completed: false,
            limited_read_cut_level_b_completed: false,
            production_observation_level_b_completed: false,
        }
    }

    fn complete_level_b_evidence() -> SqliteStopWriteLevelBEvidence {
        SqliteStopWriteLevelBEvidence {
            production_db_apply_level_b_completed: true,
            limited_read_cut_level_b_completed: true,
            production_observation_level_b_completed: true,
        }
    }

    fn fixture_dir(name: &str) -> PathBuf {
        r3_a12_fixture_root().join(name)
    }

    fn temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("r3-a12-{label}-{nanos}"))
    }
}
