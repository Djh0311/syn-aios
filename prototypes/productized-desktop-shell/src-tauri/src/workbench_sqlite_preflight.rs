use crate::utils::hash::{
    sha256_hex_bytes as sha256_hex, workbench_source_aggregate_hash,
    WorkbenchSourceAggregateHashEntry, WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM,
};
use crate::workbench_sqlite_importer::{
    CANONICAL_RUNTIME_LOG, LEGACY_RUNTIME_LOG_ALIAS, OPTIONAL_SIDECARS, PRIMARY_WORKFLOW_STATE,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const DENIED_PATH_MARKERS: &[&str] = &[
    "/users/yoyi/.codex",
    ".codex",
    ".env",
    "token",
    "secret",
    "credential",
    "keychain",
    "oauth",
    "provider_credential",
    "full_transcript",
    "rollout",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SqliteProductionPreflightConfig {
    pub(crate) primary_workflow_state: String,
    pub(crate) allowed_sidecars: BTreeSet<String>,
    pub(crate) denied_path_markers: Vec<String>,
}

impl Default for SqliteProductionPreflightConfig {
    fn default() -> Self {
        Self {
            primary_workflow_state: PRIMARY_WORKFLOW_STATE.to_string(),
            allowed_sidecars: OPTIONAL_SIDECARS
                .iter()
                .map(|sidecar| (*sidecar).to_string())
                .collect(),
            denied_path_markers: DENIED_PATH_MARKERS
                .iter()
                .map(|marker| (*marker).to_string())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProductionPreflightReport {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) status: String,
    pub(crate) source_root_ref: String,
    pub(crate) source_root_hash: String,
    pub(crate) source_root_hash_algorithm: String,
    pub(crate) files: Vec<SqlitePreflightFileReport>,
    pub(crate) counts: SqlitePreflightCounts,
    pub(crate) backup_readiness: SqlitePreflightBackupReadiness,
    pub(crate) sidecar_readiness: Vec<SqlitePreflightSidecarReadiness>,
    pub(crate) warnings: Vec<String>,
    pub(crate) blocked_reasons: Vec<String>,
    pub(crate) production_db_created: bool,
    pub(crate) production_root_written: bool,
    pub(crate) read_cut_enabled: bool,
    pub(crate) stop_write_json: bool,
    pub(crate) codex_home_touched: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqlitePreflightFileReport {
    pub(crate) path_ref: String,
    pub(crate) path_hash: String,
    pub(crate) file_hash: Option<String>,
    pub(crate) size_bytes: u64,
    pub(crate) schema_version: Option<String>,
    pub(crate) revision: Option<i64>,
    pub(crate) top_level_keys: Vec<String>,
    pub(crate) record_count_estimate: usize,
    pub(crate) redaction_status: String,
    pub(crate) classification: String,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqlitePreflightCounts {
    pub(crate) files_seen: usize,
    pub(crate) files_accepted: usize,
    pub(crate) files_missing_optional: usize,
    pub(crate) files_rejected: usize,
    pub(crate) warnings: usize,
    pub(crate) blocked_reasons: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqlitePreflightBackupReadiness {
    pub(crate) backups_dir_present: bool,
    pub(crate) workflow_state_backup_count: usize,
    pub(crate) latest_workflow_state_backup_ref: Option<String>,
    pub(crate) latest_workflow_state_backup_hash: Option<String>,
    pub(crate) latest_workflow_state_backup_timestamp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqlitePreflightSidecarReadiness {
    pub(crate) sidecar_name: String,
    pub(crate) status: String,
    pub(crate) canonical: bool,
}

pub(crate) fn scan_workbench_state_root_preflight(
    source_root: &Path,
    report_path: Option<&Path>,
) -> Result<SqliteProductionPreflightReport, String> {
    scan_workbench_state_root_preflight_with_config(
        source_root,
        report_path,
        &SqliteProductionPreflightConfig::default(),
    )
}

pub(crate) fn scan_workbench_state_root_preflight_with_config(
    source_root: &Path,
    report_path: Option<&Path>,
    config: &SqliteProductionPreflightConfig,
) -> Result<SqliteProductionPreflightReport, String> {
    let denied_path_markers = effective_denied_path_markers(config);
    validate_source_root(source_root, &denied_path_markers)?;
    if let Some(path) = report_path {
        validate_report_path(path, source_root, &denied_path_markers)?;
    }

    let mut file_reports = Vec::new();
    let mut warnings = Vec::new();
    let mut blocked_reasons = Vec::new();

    let mut entries = fs::read_dir(source_root)
        .map_err(|error| {
            format!(
                "preflight_source_root_unreadable:{}:{error}",
                source_root.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("preflight_source_root_entry_unreadable:{error}"))?;
    entries.sort_by_key(|entry| entry.file_name());

    let allowed_names = allowed_file_names(config);
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if denied_path_hit(&path, &denied_path_markers)
            || denied_name_hit(&name, &denied_path_markers)
        {
            let report = rejected_path_report(source_root, &path, "denied_path_or_name")?;
            blocked_reasons.push(format!("{} rejected", report.path_ref));
            file_reports.push(report);
            continue;
        }
        if path.is_dir() {
            if name != "backups" {
                warnings.push(format!("directory_ignored:{name}"));
            }
            continue;
        }
        if is_ignored_support_file(&name) {
            continue;
        }
        let report = scan_file(
            source_root,
            &path,
            &name,
            &allowed_names,
            &denied_path_markers,
        )?;
        if report.classification == "rejected" {
            blocked_reasons.push(format!("{} rejected", report.path_ref));
        }
        file_reports.push(report);
    }

    let present_names = file_reports
        .iter()
        .filter(|file| file.classification == "accepted")
        .map(|file| file.path_ref.as_str())
        .collect::<BTreeSet<_>>();
    let sidecar_readiness = sidecar_readiness(&present_names, &config.allowed_sidecars);
    for readiness in &sidecar_readiness {
        if readiness.status == "missing_optional" {
            warnings.push(format!("missing_optional:{}", readiness.sidecar_name));
        }
    }

    let backup_readiness = scan_backup_readiness(source_root, &denied_path_markers)?;
    if !backup_readiness.backups_dir_present {
        warnings.push("backup_readiness:backups_dir_missing".to_string());
    }

    let source_root_hash = source_root_hash(&file_reports);
    let mut counts = SqlitePreflightCounts {
        files_seen: file_reports.len(),
        files_accepted: file_reports
            .iter()
            .filter(|file| file.classification == "accepted")
            .count(),
        files_missing_optional: sidecar_readiness
            .iter()
            .filter(|sidecar| sidecar.status == "missing_optional")
            .count(),
        files_rejected: file_reports
            .iter()
            .filter(|file| file.classification == "rejected")
            .count(),
        warnings: 0,
        blocked_reasons: 0,
    };
    counts.warnings = warnings.len()
        + file_reports
            .iter()
            .map(|file| file.warnings.len())
            .sum::<usize>();
    counts.blocked_reasons = blocked_reasons.len();

    let status = if blocked_reasons.is_empty() {
        "preflight_ready"
    } else {
        "preflight_blocked"
    };
    let report = SqliteProductionPreflightReport {
        schema_version: "workbench_sqlite_production_preflight.v1".to_string(),
        mode: "production_preflight".to_string(),
        status: status.to_string(),
        source_root_ref: source_root.display().to_string(),
        source_root_hash,
        source_root_hash_algorithm: WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM.to_string(),
        files: file_reports,
        counts,
        backup_readiness,
        sidecar_readiness,
        warnings,
        blocked_reasons,
        production_db_created: false,
        production_root_written: false,
        read_cut_enabled: false,
        stop_write_json: false,
        codex_home_touched: false,
    };

    if let Some(path) = report_path {
        write_report(path, &report)?;
    }
    Ok(report)
}

fn scan_file(
    source_root: &Path,
    path: &Path,
    name: &str,
    allowed_names: &BTreeSet<String>,
    denied_path_markers: &[String],
) -> Result<SqlitePreflightFileReport, String> {
    let path_ref = path_ref(source_root, path)?;
    let size_bytes = fs::metadata(path)
        .map_err(|error| format!("preflight_file_metadata_failed:{}:{error}", path.display()))?
        .len();
    let mut warnings = Vec::new();
    if denied_path_hit(path, denied_path_markers) || denied_name_hit(name, denied_path_markers) {
        return Ok(rejected_file(path_ref, size_bytes, "denied_path_or_name"));
    }
    if path.extension().and_then(|value| value.to_str()) != Some("json") {
        return Ok(rejected_file(path_ref, size_bytes, "non_json_file"));
    }
    if !allowed_names.contains(name) && !is_workflow_state_backup(name) {
        return Ok(rejected_file(path_ref, size_bytes, "unknown_json_file"));
    }

    let bytes = fs::read(path)
        .map_err(|error| format!("preflight_file_read_failed:{}:{error}", path.display()))?;
    let file_hash = sha256_hex(&bytes);
    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(format!("json_parse_failed:{error}"));
            return Ok(SqlitePreflightFileReport {
                path_hash: sha256_hex(path_ref.as_bytes()),
                path_ref,
                file_hash: Some(file_hash),
                size_bytes,
                schema_version: None,
                revision: None,
                top_level_keys: Vec::new(),
                record_count_estimate: 0,
                redaction_status: "metadata_only_parse_failed".to_string(),
                classification: "rejected".to_string(),
                warnings,
            });
        }
    };
    if contains_forbidden_top_level_key(&value, denied_path_markers) {
        return Ok(SqlitePreflightFileReport {
            path_hash: sha256_hex(path_ref.as_bytes()),
            path_ref,
            file_hash: Some(file_hash),
            size_bytes,
            schema_version: schema_version(&value),
            revision: revision(&value),
            top_level_keys: top_level_keys(&value),
            record_count_estimate: record_count_estimate(&value),
            redaction_status: "blocked_forbidden_top_level_key".to_string(),
            classification: "rejected".to_string(),
            warnings: vec!["forbidden_top_level_key".to_string()],
        });
    }
    Ok(SqlitePreflightFileReport {
        path_hash: sha256_hex(path_ref.as_bytes()),
        path_ref,
        file_hash: Some(file_hash),
        size_bytes,
        schema_version: schema_version(&value),
        revision: revision(&value),
        top_level_keys: top_level_keys(&value),
        record_count_estimate: record_count_estimate(&value),
        redaction_status: "metadata_hash_schema_revision_counts_only".to_string(),
        classification: "accepted".to_string(),
        warnings,
    })
}

fn rejected_file(path_ref: String, size_bytes: u64, reason: &str) -> SqlitePreflightFileReport {
    SqlitePreflightFileReport {
        path_hash: sha256_hex(path_ref.as_bytes()),
        path_ref,
        file_hash: None,
        size_bytes,
        schema_version: None,
        revision: None,
        top_level_keys: Vec::new(),
        record_count_estimate: 0,
        redaction_status: "metadata_only_rejected_without_body_read".to_string(),
        classification: "rejected".to_string(),
        warnings: vec![reason.to_string()],
    }
}

fn validate_source_root(path: &Path, denied_path_markers: &[String]) -> Result<(), String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!(
            "preflight_source_root_required: {}",
            path.display()
        ));
    }
    if denied_path_hit(path, denied_path_markers) {
        return Err(format!("preflight_source_root_denied: {}", path.display()));
    }
    Ok(())
}

fn validate_report_path(
    path: &Path,
    source_root: &Path,
    denied_path_markers: &[String],
) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!(
            "preflight_report_path_must_be_absolute: {}",
            path.display()
        ));
    }
    if denied_path_hit(path, denied_path_markers) {
        return Err(format!("preflight_report_path_denied: {}", path.display()));
    }
    if path.starts_with(source_root) {
        return Err(format!(
            "preflight_report_path_inside_source_root_denied: {}",
            path.display()
        ));
    }
    Ok(())
}

fn allowed_file_names(config: &SqliteProductionPreflightConfig) -> BTreeSet<String> {
    let mut allowed = BTreeSet::from([config.primary_workflow_state.clone()]);
    for sidecar in &config.allowed_sidecars {
        allowed.insert(sidecar.clone());
    }
    allowed
}

fn effective_denied_path_markers(config: &SqliteProductionPreflightConfig) -> Vec<String> {
    let mut markers = DENIED_PATH_MARKERS
        .iter()
        .map(|marker| (*marker).to_string())
        .collect::<BTreeSet<_>>();
    markers.extend(config.denied_path_markers.iter().cloned());
    markers.into_iter().collect()
}

fn sidecar_readiness(
    present_names: &BTreeSet<&str>,
    allowed_sidecars: &BTreeSet<String>,
) -> Vec<SqlitePreflightSidecarReadiness> {
    allowed_sidecars
        .iter()
        .map(|sidecar| SqlitePreflightSidecarReadiness {
            sidecar_name: sidecar.clone(),
            status: if present_names.contains(sidecar.as_str()) {
                "present"
            } else {
                "missing_optional"
            }
            .to_string(),
            canonical: sidecar != LEGACY_RUNTIME_LOG_ALIAS,
        })
        .collect()
}

fn scan_backup_readiness(
    source_root: &Path,
    denied_path_markers: &[String],
) -> Result<SqlitePreflightBackupReadiness, String> {
    let backups_dir = source_root.join("backups");
    if !backups_dir.is_dir() {
        return Ok(SqlitePreflightBackupReadiness::default());
    }
    let mut backups = fs::read_dir(&backups_dir)
        .map_err(|error| {
            format!(
                "preflight_backup_dir_unreadable:{}:{error}",
                backups_dir.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("preflight_backup_entry_unreadable:{error}"))?;
    backups.sort_by_key(|entry| entry.file_name());
    let mut latest_ref = None;
    let mut latest_hash = None;
    let mut latest_timestamp = None;
    let mut count = 0;
    for entry in backups {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !path.is_file()
            || !is_workflow_state_backup(&name)
            || denied_name_hit(&name, denied_path_markers)
        {
            continue;
        }
        count += 1;
        let bytes = fs::read(&path)
            .map_err(|error| format!("preflight_backup_read_failed:{}:{error}", path.display()))?;
        latest_ref = Some(format!("backups/{name}"));
        latest_hash = Some(sha256_hex(&bytes));
        latest_timestamp = backup_timestamp(&name);
    }
    Ok(SqlitePreflightBackupReadiness {
        backups_dir_present: true,
        workflow_state_backup_count: count,
        latest_workflow_state_backup_ref: latest_ref,
        latest_workflow_state_backup_hash: latest_hash,
        latest_workflow_state_backup_timestamp: latest_timestamp,
    })
}

fn is_ignored_support_file(name: &str) -> bool {
    name.starts_with('.') || name.ends_with(".tmp") || name.ends_with(".lock")
}

fn is_workflow_state_backup(name: &str) -> bool {
    name.starts_with("workflow-state.v0.") && name.ends_with(".json")
}

fn backup_timestamp(name: &str) -> Option<String> {
    name.strip_prefix("workflow-state.v0.")
        .and_then(|value| value.strip_suffix(".json"))
        .map(ToString::to_string)
}

fn path_ref(source_root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(source_root)
        .map(|value| value.to_string_lossy().to_string())
        .map_err(|error| format!("preflight_path_ref_failed:{}:{error}", path.display()))
}

fn schema_version(value: &Value) -> Option<String> {
    value
        .get("schema_version")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn revision(value: &Value) -> Option<i64> {
    value.get("revision").and_then(Value::as_i64)
}

fn top_level_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
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

fn contains_forbidden_top_level_key(value: &Value, denied_path_markers: &[String]) -> bool {
    value
        .as_object()
        .map(|object| {
            object
                .keys()
                .any(|key| denied_name_hit(key, denied_path_markers))
        })
        .unwrap_or(false)
}

fn source_root_hash(files: &[SqlitePreflightFileReport]) -> String {
    workbench_source_aggregate_hash(files.iter().map(|file| WorkbenchSourceAggregateHashEntry {
        path_ref: &file.path_ref,
        file_hash: file.file_hash.as_deref(),
        classification: &file.classification,
    }))
}

fn denied_path_hit(path: &Path, denied_path_markers: &[String]) -> bool {
    let normalized = path.to_string_lossy().to_lowercase();
    denied_path_markers
        .iter()
        .any(|marker| normalized.contains(&marker.to_lowercase()))
}

fn denied_name_hit(name: &str, denied_path_markers: &[String]) -> bool {
    let lower = name.to_lowercase();
    denied_path_markers
        .iter()
        .any(|marker| lower.contains(marker.trim_matches('/').to_lowercase().as_str()))
}

fn write_report(path: &Path, report: &SqliteProductionPreflightReport) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("preflight_report_path_has_no_parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "preflight_report_parent_create_failed:{}:{error}",
            parent.display()
        )
    })?;
    let bytes = serde_json::to_vec(&json!(report))
        .map_err(|error| format!("preflight_report_serialize_failed:{error}"))?;
    fs::write(path, bytes)
        .map_err(|error| format!("preflight_report_write_failed:{}:{error}", path.display()))
}

fn rejected_path_report(
    source_root: &Path,
    path: &Path,
    reason: &str,
) -> Result<SqlitePreflightFileReport, String> {
    let path_ref = path_ref(source_root, path)?;
    let size_bytes = fs::metadata(path)
        .map_err(|error| format!("preflight_path_metadata_failed:{}:{error}", path.display()))?
        .len();
    Ok(rejected_file(path_ref, size_bytes, reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_preflight_scans_valid_root_without_writing_production_flags() {
        let root = temp_test_dir("valid");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({
                "schema_version": "workflow_state_v0",
                "revision": 7,
                "projects": [],
                "workflows": [],
                "nodes": [],
                "edges": []
            }),
        );
        write_json(
            &root.join(CANONICAL_RUNTIME_LOG),
            &json!({
                "schema_version": "runtime_log_store.v1",
                "revision": 1,
                "entries": []
            }),
        );
        fs::create_dir_all(root.join("backups")).expect("backup dir");
        write_json(
            &root.join("backups/workflow-state.v0.2026-06-11T00-00-00.json"),
            &json!({"schema_version": "workflow_state_v0", "revision": 6}),
        );
        let report_path = temp_test_dir("valid-report").join("preflight.json");

        let report = scan_workbench_state_root_preflight(&root, Some(&report_path))
            .expect("preflight should scan");

        assert_eq!(report.status, "preflight_ready");
        assert!(report.report_flags_stay_false());
        assert!(report.report_contains_file(PRIMARY_WORKFLOW_STATE));
        assert_eq!(report.backup_readiness.workflow_state_backup_count, 1);
        assert!(report_path.exists());
    }

    #[test]
    fn sqlite_preflight_missing_optional_sidecars_are_warnings_not_blockers() {
        let root = temp_test_dir("missing-optional");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({"schema_version": "workflow_state_v0", "revision": 1, "projects": []}),
        );

        let report =
            scan_workbench_state_root_preflight(&root, None).expect("preflight should scan");

        assert_eq!(report.status, "preflight_ready");
        assert!(report.counts.files_missing_optional > 0);
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("missing_optional")));
    }

    #[test]
    fn sqlite_preflight_source_root_hash_uses_canonical_aggregate_helper() {
        let root = temp_test_dir("canonical-aggregate");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({
                "schema_version": "workflow_state_v0",
                "revision": 1,
                "projects": []
            }),
        );
        write_json(
            &root.join("plan-authorizations.v1.json"),
            &json!({
                "schema_version": "plan_authorization_store.v1",
                "revision": 2,
                "authorizations": [],
                "audit_events": []
            }),
        );

        let report =
            scan_workbench_state_root_preflight(&root, None).expect("preflight should scan");
        let canonical_hash = workbench_source_aggregate_hash(report.files.iter().map(|file| {
            WorkbenchSourceAggregateHashEntry {
                path_ref: &file.path_ref,
                file_hash: file.file_hash.as_deref(),
                classification: &file.classification,
            }
        }));

        assert_eq!(
            report.source_root_hash_algorithm,
            WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM
        );
        assert_eq!(report.source_root_hash, canonical_hash);
    }

    #[test]
    fn sqlite_preflight_unknown_json_blocks_without_body_output() {
        let root = temp_test_dir("unknown-json");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({"schema_version": "workflow_state_v0", "revision": 1}),
        );
        write_json(
            &root.join("unexpected-store.json"),
            &json!({"schema_version": "unknown", "items": [{"body": "do not output"}]}),
        );

        let report =
            scan_workbench_state_root_preflight(&root, None).expect("preflight should scan");

        assert_eq!(report.status, "preflight_blocked");
        let rejected = report
            .files
            .iter()
            .find(|file| file.path_ref == "unexpected-store.json")
            .expect("unknown file report");
        assert_eq!(rejected.classification, "rejected");
        assert!(rejected.file_hash.is_none());
        assert!(serde_json::to_string(&report)
            .expect("serialize")
            .find("do not output")
            .is_none());
    }

    #[test]
    fn sqlite_preflight_denied_sensitive_name_blocks_without_reading_body() {
        let root = temp_test_dir("denied-sensitive");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({"schema_version": "workflow_state_v0", "revision": 1}),
        );
        fs::write(
            root.join("provider-credential.json"),
            b"{\"secret\":\"value\"}",
        )
        .expect("write denied file");

        let report =
            scan_workbench_state_root_preflight(&root, None).expect("preflight should scan");

        assert_eq!(report.status, "preflight_blocked");
        let serialized = serde_json::to_string(&report).expect("serialize");
        assert!(!serialized.contains("value"));
        assert!(report
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("provider-credential.json")));
    }

    #[test]
    fn sqlite_preflight_denies_codex_home_root() {
        let err = scan_workbench_state_root_preflight(Path::new("/Users/yoyi/.codex"), None)
            .expect_err("codex home must be denied");

        assert!(err.contains("preflight_source_root_denied"));
    }

    #[test]
    fn sqlite_preflight_denied_dotfile_and_directory_block_before_ignore() {
        let root = temp_test_dir("denied-dotfile-dir");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({"schema_version": "workflow_state_v0", "revision": 1}),
        );
        fs::write(root.join(".env"), b"SECRET=value").expect("write denied dotfile");
        fs::create_dir_all(root.join(".codex")).expect("write denied dir");

        let report =
            scan_workbench_state_root_preflight(&root, None).expect("preflight should scan");

        assert_eq!(report.status, "preflight_blocked");
        assert!(report
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains(".env")));
        assert!(report
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains(".codex")));
        let serialized = serde_json::to_string(&report).expect("serialize");
        assert!(!serialized.contains("SECRET=value"));
    }

    #[test]
    fn sqlite_preflight_denies_report_path_inside_source_root() {
        let root = temp_test_dir("report-inside-source");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({"schema_version": "workflow_state_v0", "revision": 1}),
        );

        let err =
            scan_workbench_state_root_preflight(&root, Some(&root.join("reports/preflight.json")))
                .expect_err("report path inside source root should be denied");

        assert!(err.contains("preflight_report_path_inside_source_root_denied"));
    }

    #[test]
    fn sqlite_preflight_accepts_explicit_sidecar_list_and_denies_explicit_markers() {
        let root = temp_test_dir("explicit-config");
        write_json(
            &root.join(PRIMARY_WORKFLOW_STATE),
            &json!({"schema_version": "workflow_state_v0", "revision": 1}),
        );
        write_json(
            &root.join("custom-allowed.json"),
            &json!({
                "schema_version": "custom_sidecar.v1",
                "revision": 2,
                "items": []
            }),
        );
        write_json(
            &root.join("custom-blocked.json"),
            &json!({
                "schema_version": "custom_sidecar.v1",
                "business_body": "do not output"
            }),
        );
        let mut config = SqliteProductionPreflightConfig::default();
        config
            .allowed_sidecars
            .insert("custom-allowed.json".to_string());
        config
            .allowed_sidecars
            .insert("custom-blocked.json".to_string());
        config.denied_path_markers.push("business_body".to_string());

        let report = scan_workbench_state_root_preflight_with_config(&root, None, &config)
            .expect("preflight should scan explicit config");

        assert_eq!(report.status, "preflight_blocked");
        assert!(report.report_contains_file("custom-allowed.json"));
        let blocked = report
            .files
            .iter()
            .find(|file| file.path_ref == "custom-blocked.json")
            .expect("blocked custom file report");
        assert_eq!(blocked.classification, "rejected");
        assert_eq!(blocked.redaction_status, "blocked_forbidden_top_level_key");
        assert!(serde_json::to_string(&report)
            .expect("serialize")
            .find("do not output")
            .is_none());
    }

    #[test]
    #[ignore = "requires explicit R3 B0 hash calibration authorization and real workbench state root"]
    fn r3_b0_hash_calibration_real_workbench_state_root_requires_env_authorization() {
        let confirmation = std::env::var("R3_B0_HASH_CALIBRATION_CONFIRM")
            .expect("R3_B0_HASH_CALIBRATION_CONFIRM is required for real B0 hash calibration");
        assert_eq!(confirmation, "CONFIRMED_READONLY_2026_06_15");
        let source = canonical_env_path("R3_B0_SOURCE_STATE_ROOT");
        let report_path = canonical_parent_env_path("R3_B0_PREFLIGHT_REPORT_PATH");
        let expected_workflow_hash = std::env::var("R3_B0_EXPECTED_WORKFLOW_STATE_HASH")
            .expect("R3_B0_EXPECTED_WORKFLOW_STATE_HASH is required");
        let expected_authorization_hash = std::env::var("R3_B0_EXPECTED_PLAN_AUTHORIZATIONS_HASH")
            .expect("R3_B0_EXPECTED_PLAN_AUTHORIZATIONS_HASH is required");

        let report = scan_workbench_state_root_preflight(&source, Some(&report_path))
            .expect("B0 hash calibration preflight must complete");
        let workflow = report
            .files
            .iter()
            .find(|file| file.path_ref == PRIMARY_WORKFLOW_STATE)
            .expect("workflow state file must be present");
        let plan_authorizations = report
            .files
            .iter()
            .find(|file| file.path_ref == "plan-authorizations.v1.json")
            .expect("plan authorizations file must be present");

        assert_eq!(
            workflow.file_hash.as_deref(),
            Some(expected_workflow_hash.as_str())
        );
        assert_eq!(
            plan_authorizations.file_hash.as_deref(),
            Some(expected_authorization_hash.as_str())
        );
        assert_eq!(report.counts.files_seen, 2);
        assert_eq!(report.counts.files_accepted, 2);
        assert_eq!(report.counts.files_rejected, 0);
        assert_eq!(report.counts.blocked_reasons, 0);
        assert_eq!(
            report.source_root_hash_algorithm,
            WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM
        );
        println!(
            "R3_B0_CANONICAL_SOURCE_ROOT_HASH={}",
            report.source_root_hash
        );
        println!(
            "R3_B0_SOURCE_ROOT_HASH_ALGORITHM={}",
            report.source_root_hash_algorithm
        );
        println!("R3_B0_PREFLIGHT_REPORT_PATH={}", report_path.display());
    }

    trait ReportAssertions {
        fn report_flags_stay_false(&self) -> bool;
        fn report_contains_file(&self, path_ref: &str) -> bool;
    }

    impl ReportAssertions for SqliteProductionPreflightReport {
        fn report_flags_stay_false(&self) -> bool {
            !self.production_db_created
                && !self.production_root_written
                && !self.read_cut_enabled
                && !self.stop_write_json
                && !self.codex_home_touched
        }

        fn report_contains_file(&self, path_ref: &str) -> bool {
            self.files.iter().any(|file| file.path_ref == path_ref)
        }
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("r3-a7-{label}-{nonce}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
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

    fn write_json(path: &Path, value: &Value) {
        let parent = path.parent().expect("parent");
        fs::create_dir_all(parent).expect("parent dir");
        fs::write(path, serde_json::to_vec(value).expect("json")).expect("write json");
    }
}
