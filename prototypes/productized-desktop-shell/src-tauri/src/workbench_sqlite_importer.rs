use crate::utils::hash::sha256_hex_bytes as sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(crate) const WORKBENCH_SQLITE_IMPORTER_VERSION: &str = "workbench_sqlite_importer_dry_run_v0";
pub(crate) const PRIMARY_WORKFLOW_STATE: &str = "workflow-state.v0.json";
pub(crate) const CANONICAL_RUNTIME_LOG: &str = "runtime-logs.v1.json";
pub(crate) const LEGACY_RUNTIME_LOG_ALIAS: &str = "runtime-log.v1.json";
pub(crate) const OPTIONAL_SIDECARS: &[&str] = &[
    "blackboard-candidates.v1.json",
    "formal-memories.v1.json",
    "memory-candidates.v1.json",
    "memory-capture-events.v1.json",
    "memory-entity-relations.v1.json",
    "memory-lint.v1.json",
    "memory-patterns.v1.json",
    "observations.v1.json",
    "plan-authorizations.v1.json",
    "project-proposals.v1.json",
    "real-execution-product-commands.v1.json",
    // M1 completeness (2026-07-13): three supervisor ledgers (previously rejected_unknown).
    "global-supervisor-reviews.v1.json",
    "supervisor-action-control.v1.json",
    "supervisor-orchestrator.v1.json",
    LEGACY_RUNTIME_LOG_ALIAS,
    CANONICAL_RUNTIME_LOG,
    "session-continuations.v1.json",
];
const WORKFLOW_ARRAYS: &[(&str, &[&str])] = &[
    ("projects", &["project_id", "id"]),
    ("agent_adapters", &["adapter_id", "id"]),
    ("workflows", &["workflow_id", "id"]),
    ("nodes", &["node_id", "id"]),
    ("edges", &["edge_id", "id"]),
    ("work_items", &["work_item_id", "id"]),
    ("artifacts", &["artifact_id", "id"]),
    ("reviews", &["review_id", "id"]),
    ("audit_events", &["event_id", "audit_event_id", "id"]),
    ("capabilities", &["capability_id", "id"]),
    ("harness_resources", &["resource_id", "id"]),
    (
        "workflow_node_session_bindings",
        &["binding_id", "session_id", "id"],
    ),
    ("workflow_node_dispatches", &["dispatch_id", "id"]),
];
// M1 completeness (2026-07-13): main-store top-level arrays that were never collected (layer c).
// Collected when present but NOT required — an empty/new workflow legitimately has none, so they
// are intentionally excluded from validate_primary_workflow to avoid spurious "missing" warnings.
// workflow_machine_runs is dead-in-code / unknown-provenance (M0 §three R1) — collected for
// round-trip preservation only, never re-armed to a live writer.
const WORKFLOW_OPTIONAL_ARRAYS: &[(&str, &[&str])] = &[
    ("execution_attempts", &["attempt_id", "id"]),
    ("permission_requests", &["request_id", "id"]),
    ("workflow_chain_runs", &["chain_run_id", "id"]),
    ("workflow_execution_controls", &["control_id", "id"]),
    ("workflow_machine_runs", &["run_id", "id"]),
];
const SENSITIVE_KEY_PARTS: &[&str] = &[
    "prompt_body",
    "secret",
    "token",
    "credential",
    "keychain",
    "oauth",
    "provider_credential",
    "full_transcript",
    "transcript_body",
    "rollout_body",
];
const SENSITIVE_STRING_MARKERS: &[&str] = &[
    "provider credential",
    "full transcript",
    "rollout body",
    "prompt_body",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteImportDryRunReport {
    pub(crate) batch_id: String,
    pub(crate) mode: String,
    pub(crate) batch_status: String,
    pub(crate) importer_version: String,
    pub(crate) source_root_ref: String,
    pub(crate) source_root_hash: String,
    pub(crate) source_inventory: Vec<SqliteImportSourceReport>,
    pub(crate) record_summaries: Vec<SqliteImportRecordSummary>,
    pub(crate) counts: SqliteImportDryRunCounts,
    pub(crate) warnings: Vec<String>,
    pub(crate) conflicts: Vec<String>,
    pub(crate) runtime_log_alias: RuntimeLogAliasReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteImportSourceReport {
    pub(crate) source_kind: String,
    pub(crate) source_path: String,
    pub(crate) source_path_hash: String,
    pub(crate) source_hash: Option<String>,
    pub(crate) source_schema_version: Option<String>,
    pub(crate) detected_revision: Option<i64>,
    pub(crate) classification: String,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteImportRecordSummary {
    pub(crate) source_kind: String,
    pub(crate) record_kind: String,
    pub(crate) natural_key: String,
    pub(crate) record_hash: String,
    pub(crate) classification: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteImportDryRunCounts {
    pub(crate) files_seen: usize,
    pub(crate) files_accepted: usize,
    pub(crate) files_missing_optional: usize,
    pub(crate) files_rejected: usize,
    pub(crate) proposed_inserts: usize,
    pub(crate) proposed_updates: usize,
    pub(crate) skips: usize,
    pub(crate) conflicts: usize,
    pub(crate) warnings: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuntimeLogAliasReport {
    pub(crate) canonical_present: bool,
    pub(crate) legacy_alias_present: bool,
    pub(crate) canonical_source_kind: Option<String>,
    pub(crate) legacy_source_kind: Option<String>,
    pub(crate) policy: String,
}

pub(crate) fn dry_run_import_fixture_dir(_root: &Path) -> Result<SqliteImportDryRunReport, String> {
    dry_run_import_fixture_dir_with_previous(_root, None)
}

pub(crate) fn dry_run_import_fixture_dir_with_previous(
    root: &Path,
    previous: Option<&SqliteImportDryRunReport>,
) -> Result<SqliteImportDryRunReport, String> {
    if !root.is_dir() {
        return Err(format!("fixture_dir_missing: {}", root.display()));
    }

    let mut source_inventory = Vec::new();
    let mut record_summaries = Vec::new();
    let mut warnings = Vec::new();
    let mut conflicts = Vec::new();
    let mut batch_status = "accepted".to_string();
    let mut previous_records = BTreeMap::new();
    if let Some(previous) = previous {
        for record in &previous.record_summaries {
            previous_records.insert(
                (
                    record.source_kind.clone(),
                    record.record_kind.clone(),
                    record.natural_key.clone(),
                ),
                record.record_hash.clone(),
            );
        }
    }

    let primary_path = root.join(PRIMARY_WORKFLOW_STATE);
    if !primary_path.exists() {
        source_inventory.push(missing_optional_source(PRIMARY_WORKFLOW_STATE, root));
        return Ok(finalize_report(
            root,
            "rejected_missing_primary".to_string(),
            source_inventory,
            record_summaries,
            warnings,
            vec!["missing required workflow-state.v0.json".to_string()],
        ));
    }

    let primary_read = read_json_source(&primary_path, PRIMARY_WORKFLOW_STATE);
    match primary_read {
        SourceRead::Accepted(source, value) => {
            if contains_sensitive_value(&value) {
                let mut rejected = source;
                rejected.classification = "rejected_sensitive".to_string();
                source_inventory.push(rejected);
                return Ok(finalize_report(
                    root,
                    "rejected_sensitive".to_string(),
                    source_inventory,
                    record_summaries,
                    warnings,
                    conflicts,
                ));
            }
            for warning in validate_primary_workflow(&value) {
                warnings.push(warning);
            }
            collect_workflow_records(
                &mut record_summaries,
                &mut conflicts,
                &mut warnings,
                &previous_records,
                PRIMARY_WORKFLOW_STATE,
                &value,
            );
            source_inventory.push(source);
        }
        SourceRead::Rejected(source, status) => {
            source_inventory.push(source);
            return Ok(finalize_report(
                root,
                status,
                source_inventory,
                record_summaries,
                warnings,
                conflicts,
            ));
        }
    }

    let allowed = OPTIONAL_SIDECARS.iter().copied().collect::<BTreeSet<_>>();
    for name in OPTIONAL_SIDECARS {
        let path = root.join(name);
        if !path.exists() {
            source_inventory.push(missing_optional_source(name, root));
            continue;
        }
        match read_json_source(&path, name) {
            SourceRead::Accepted(mut source, value) => {
                if contains_sensitive_value(&value) {
                    source.classification = "rejected_sensitive".to_string();
                    source_inventory.push(source);
                    batch_status = "rejected_sensitive".to_string();
                    continue;
                }
                collect_sidecar_records(
                    &mut record_summaries,
                    &mut conflicts,
                    &mut warnings,
                    &previous_records,
                    name,
                    &value,
                );
                source_inventory.push(source);
            }
            SourceRead::Rejected(source, status) => {
                if status == "rejected_corrupt" {
                    warnings.push(format!("optional sidecar corrupt: {name}"));
                }
                source_inventory.push(source);
            }
        }
    }

    for entry in sorted_fixture_entries(root)? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == PRIMARY_WORKFLOW_STATE || allowed.contains(name.as_str()) {
            continue;
        }
        if name.ends_with(".v1.json") {
            source_inventory.push(SqliteImportSourceReport {
                source_kind: source_kind_for_name(&name).to_string(),
                source_path: name.clone(),
                source_path_hash: sha256_hex(name.as_bytes()),
                source_hash: fs::read(&path).ok().map(|bytes| sha256_hex(&bytes)),
                source_schema_version: None,
                detected_revision: None,
                classification: "rejected_unknown".to_string(),
                warnings: vec!["unknown_sidecar_requires_supervisor_decision".to_string()],
            });
            warnings.push(format!("unknown sidecar rejected: {name}"));
            if batch_status == "accepted" {
                batch_status = "accepted_with_rejections".to_string();
            }
        }
    }

    if !conflicts.is_empty() {
        batch_status = "conflict".to_string();
    } else if batch_status == "accepted"
        && source_inventory.iter().any(|source| {
            source.classification == "rejected_sensitive"
                || source.classification == "rejected_unknown"
                || source.classification == "rejected_corrupt"
        })
    {
        batch_status = "accepted_with_rejections".to_string();
    }

    Ok(finalize_report(
        root,
        batch_status,
        source_inventory,
        record_summaries,
        warnings,
        conflicts,
    ))
}

enum SourceRead {
    Accepted(SqliteImportSourceReport, Value),
    Rejected(SqliteImportSourceReport, String),
}

fn read_json_source(path: &Path, name: &str) -> SourceRead {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return SourceRead::Rejected(
                SqliteImportSourceReport {
                    source_kind: source_kind_for_name(name).to_string(),
                    source_path: name.to_string(),
                    source_path_hash: sha256_hex(name.as_bytes()),
                    source_hash: None,
                    source_schema_version: None,
                    detected_revision: None,
                    classification: "rejected_corrupt".to_string(),
                    warnings: vec![format!("source_unreadable:{error}")],
                },
                if name == PRIMARY_WORKFLOW_STATE {
                    "rejected_corrupt_primary".to_string()
                } else {
                    "accepted_with_rejections".to_string()
                },
            );
        }
    };
    let source_hash = sha256_hex(&bytes);
    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            return SourceRead::Rejected(
                SqliteImportSourceReport {
                    source_kind: source_kind_for_name(name).to_string(),
                    source_path: name.to_string(),
                    source_path_hash: sha256_hex(name.as_bytes()),
                    source_hash: Some(source_hash),
                    source_schema_version: None,
                    detected_revision: None,
                    classification: "rejected_corrupt".to_string(),
                    warnings: vec![format!("json_parse_failed:{error}")],
                },
                if name == PRIMARY_WORKFLOW_STATE {
                    "rejected_corrupt_primary".to_string()
                } else {
                    "accepted_with_rejections".to_string()
                },
            );
        }
    };
    let source_schema_version = value
        .get("schema_version")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let detected_revision = value.get("revision").and_then(Value::as_i64);
    SourceRead::Accepted(
        SqliteImportSourceReport {
            source_kind: source_kind_for_name(name).to_string(),
            source_path: name.to_string(),
            source_path_hash: sha256_hex(name.as_bytes()),
            source_hash: Some(source_hash),
            source_schema_version,
            detected_revision,
            classification: "accepted".to_string(),
            warnings: Vec::new(),
        },
        value,
    )
}

fn missing_optional_source(name: &str, root: &Path) -> SqliteImportSourceReport {
    SqliteImportSourceReport {
        source_kind: source_kind_for_name(name).to_string(),
        source_path: root.join(name).display().to_string(),
        source_path_hash: sha256_hex(name.as_bytes()),
        source_hash: None,
        source_schema_version: None,
        detected_revision: None,
        classification: "missing_optional".to_string(),
        warnings: vec!["optional sidecar missing".to_string()],
    }
}

fn validate_primary_workflow(value: &Value) -> Vec<String> {
    let mut warnings = Vec::new();
    if value.get("schema_version").and_then(Value::as_str) != Some("workflow_state_v0") {
        warnings.push("primary schema_version is not workflow_state_v0".to_string());
    }
    if value.get("workflow_version").and_then(Value::as_i64) != Some(1) {
        warnings.push("primary workflow_version is not 1".to_string());
    }
    for (array, _) in WORKFLOW_ARRAYS {
        if !value.get(*array).and_then(Value::as_array).is_some() {
            warnings.push(format!("primary {array} missing or not array"));
        }
    }
    warnings
}

fn collect_workflow_records(
    records: &mut Vec<SqliteImportRecordSummary>,
    conflicts: &mut Vec<String>,
    warnings: &mut Vec<String>,
    previous_records: &BTreeMap<(String, String, String), String>,
    source_kind: &str,
    value: &Value,
) {
    for (array, key_candidates) in WORKFLOW_ARRAYS
        .iter()
        .chain(WORKFLOW_OPTIONAL_ARRAYS.iter())
    {
        if let Some(items) = value.get(*array).and_then(Value::as_array) {
            collect_array_records(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                array,
                key_candidates,
                items,
                None,
            );
        }
    }
}

fn collect_sidecar_records(
    records: &mut Vec<SqliteImportRecordSummary>,
    conflicts: &mut Vec<String>,
    warnings: &mut Vec<String>,
    previous_records: &BTreeMap<(String, String, String), String>,
    source_name: &str,
    value: &Value,
) {
    let source_kind = source_kind_for_name(source_name);
    match source_name {
        "formal-memories.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "records",
                        &["memory_id", "id"][..],
                        Some("formal_memory_record"),
                    ),
                    (
                        "versions",
                        &["version_id", "id"][..],
                        Some("formal_memory_version"),
                    ),
                    (
                        "audit_events",
                        &["audit_event_id", "event_id", "id"][..],
                        Some("formal_memory_audit_event"),
                    ),
                ],
            );
        }
        "memory-candidates.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "candidates",
                        &["candidate_key", "candidate_id", "id"][..],
                        Some("memory_candidate"),
                    ),
                    (
                        "events",
                        &["audit_ref_id", "event_id", "id"][..],
                        Some("memory_candidate_event"),
                    ),
                ],
            );
        }
        "observations.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "observations",
                        &["observation_key", "observation_id", "id"][..],
                        Some("observation"),
                    ),
                    (
                        "events",
                        &["audit_ref_id", "event_id", "id"][..],
                        Some("observation_event"),
                    ),
                ],
            );
        }
        "memory-capture-events.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[(
                    "events",
                    &["event_key", "capture_event_id", "id"][..],
                    Some("memory_capture_event"),
                )],
            );
        }
        "plan-authorizations.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "authorizations",
                        &["authorization_id", "id"][..],
                        Some("plan_authorization"),
                    ),
                    (
                        "audit_events",
                        &["audit_event_id", "event_id", "id"][..],
                        Some("plan_authorization_audit_event"),
                    ),
                ],
            );
        }
        "project-proposals.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "proposals",
                        &["proposal_id", "id"][..],
                        Some("project_proposal"),
                    ),
                    (
                        "decisions",
                        &["decision_id", "id"][..],
                        Some("project_proposal_decision"),
                    ),
                    (
                        "audit_events",
                        &["audit_event_id", "event_id", "id"][..],
                        Some("project_proposal_audit_event"),
                    ),
                ],
            );
        }
        "real-execution-product-commands.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "commands",
                        &["product_command_id", "command_id", "id"][..],
                        Some("product_command"),
                    ),
                    (
                        "previews",
                        &["preview_id", "id"][..],
                        Some("product_command_preview"),
                    ),
                    (
                        "decisions",
                        &["decision_id", "id"][..],
                        Some("product_command_decision"),
                    ),
                    (
                        "attempts",
                        &["attempt_id", "id"][..],
                        Some("product_command_attempt"),
                    ),
                ],
            );
        }
        "session-continuations.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "continuations",
                        &["continuation_id", "id"][..],
                        Some("session_continuation"),
                    ),
                    (
                        "attempts",
                        &["attempt_id", "id"][..],
                        Some("session_continuation_attempt"),
                    ),
                    (
                        "audit_events",
                        &["event_id", "audit_event_id", "id"][..],
                        Some("session_continuation_audit_event"),
                    ),
                ],
            );
        }
        CANONICAL_RUNTIME_LOG | LEGACY_RUNTIME_LOG_ALIAS => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "entries",
                        &["entry_id", "id"][..],
                        Some("runtime_log_entry"),
                    ),
                    (
                        "summaries",
                        &["summary_id", "id"][..],
                        Some("runtime_log_summary"),
                    ),
                ],
            );
        }
        "memory-lint.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "runs",
                        &["lint_run_id", "run_id", "id"][..],
                        Some("memory_lint_run"),
                    ),
                    (
                        "findings",
                        &["finding_id", "id"][..],
                        Some("memory_lint_finding"),
                    ),
                ],
            );
        }
        "memory-entity-relations.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[(
                    "relations",
                    &["relation_id", "id"][..],
                    Some("memory_entity_relation"),
                )],
            );
        }
        "memory-patterns.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "candidates",
                        &["candidate_id", "id"][..],
                        Some("mature_pattern_candidate"),
                    ),
                    (
                        "audit_events",
                        &["audit_event_id", "event_id", "id"][..],
                        Some("mature_pattern_audit_event"),
                    ),
                ],
            );
        }
        "blackboard-candidates.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "candidates",
                        &["candidate_key", "id"][..],
                        Some("blackboard_candidate"),
                    ),
                    (
                        "audit_events",
                        &["audit_event_id", "event_id", "id"][..],
                        Some("blackboard_candidate_audit_event"),
                    ),
                ],
            );
        }
        "global-supervisor-reviews.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "reviews",
                        &["review_id", "id"][..],
                        Some("supervisor_review"),
                    ),
                    (
                        "audit_events",
                        &["event_id", "audit_event_id", "id"][..],
                        Some("supervisor_review_audit_event"),
                    ),
                    (
                        "boundary_reviews",
                        &["review_id", "id"][..],
                        Some("supervisor_boundary_review"),
                    ),
                    (
                        "boundary_audit_events",
                        &["event_id", "audit_event_id", "id"][..],
                        Some("supervisor_boundary_audit_event"),
                    ),
                ],
            );
        }
        "supervisor-action-control.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[(
                    "actions",
                    &["action_id", "id"][..],
                    Some("supervisor_action"),
                )],
            );
        }
        "supervisor-orchestrator.v1.json" => {
            collect_named_arrays(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                value,
                &[
                    (
                        "sessions",
                        &["run_id", "id"][..],
                        Some("supervisor_orchestrator_session"),
                    ),
                    (
                        "audit_events",
                        &["event_id", "audit_event_id", "id"][..],
                        Some("supervisor_orchestrator_audit_event"),
                    ),
                ],
            );
        }
        _ => collect_fallback_sidecar_record(
            records,
            conflicts,
            previous_records,
            source_kind,
            value,
        ),
    }
}

fn collect_named_arrays(
    records: &mut Vec<SqliteImportRecordSummary>,
    conflicts: &mut Vec<String>,
    warnings: &mut Vec<String>,
    previous_records: &BTreeMap<(String, String, String), String>,
    source_kind: &str,
    value: &Value,
    specs: &[(&str, &[&str], Option<&str>)],
) {
    for (array_name, keys, record_kind) in specs {
        if let Some(items) = value.get(*array_name).and_then(Value::as_array) {
            collect_array_records(
                records,
                conflicts,
                warnings,
                previous_records,
                source_kind,
                record_kind.unwrap_or(array_name),
                keys,
                items,
                Some(*array_name),
            );
        }
    }
}

fn collect_array_records(
    records: &mut Vec<SqliteImportRecordSummary>,
    conflicts: &mut Vec<String>,
    warnings: &mut Vec<String>,
    previous_records: &BTreeMap<(String, String, String), String>,
    source_kind: &str,
    record_kind: &str,
    key_candidates: &[&str],
    items: &[Value],
    array_name: Option<&str>,
) {
    let mut seen = BTreeMap::<String, String>::new();
    for (index, item) in items.iter().enumerate() {
        let record_hash = canonical_json_hash(item);
        let natural_key = natural_key(item, key_candidates).unwrap_or_else(|| {
            format!(
                "{}:{}:{}",
                array_name.unwrap_or(record_kind),
                index,
                record_hash
            )
        });
        let mut classification = "accepted".to_string();
        if let Some(existing_hash) = seen.get(&natural_key) {
            if existing_hash == &record_hash {
                classification = "skipped_duplicate".to_string();
            } else {
                classification = "conflict".to_string();
                conflicts.push(format!(
                    "{source_kind}:{record_kind}:{natural_key}:duplicate_different_hash"
                ));
            }
        }
        if let Some(previous_hash) = previous_records.get(&(
            source_kind.to_string(),
            record_kind.to_string(),
            natural_key.clone(),
        )) {
            if previous_hash == &record_hash {
                classification = "skipped_duplicate".to_string();
            } else {
                classification = "conflict".to_string();
                conflicts.push(format!(
                    "{source_kind}:{record_kind}:{natural_key}:previous_hash_conflict"
                ));
            }
        }
        if classification == "accepted" && revision_conflict_marker(item) {
            classification = "conflict".to_string();
            conflicts.push(format!(
                "{source_kind}:{record_kind}:{natural_key}:revision_conflict"
            ));
        }
        if classification == "accepted" && natural_key.starts_with("hash:") {
            warnings.push(format!(
                "{source_kind}:{record_kind}:{natural_key}:hash_key_fallback"
            ));
        }
        seen.insert(natural_key.clone(), record_hash.clone());
        records.push(SqliteImportRecordSummary {
            source_kind: source_kind.to_string(),
            record_kind: record_kind.to_string(),
            natural_key,
            record_hash,
            classification,
        });
    }
}

fn collect_fallback_sidecar_record(
    records: &mut Vec<SqliteImportRecordSummary>,
    conflicts: &mut Vec<String>,
    previous_records: &BTreeMap<(String, String, String), String>,
    source_kind: &str,
    value: &Value,
) {
    let record_hash = canonical_json_hash(value);
    let natural_key = value
        .get("store_id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("hash:{record_hash}"));
    let mut classification = "accepted".to_string();
    if let Some(previous_hash) = previous_records.get(&(
        source_kind.to_string(),
        "sidecar_store".to_string(),
        natural_key.clone(),
    )) {
        if previous_hash == &record_hash {
            classification = "skipped_duplicate".to_string();
        } else {
            classification = "conflict".to_string();
            conflicts.push(format!(
                "{source_kind}:sidecar_store:{natural_key}:previous_hash_conflict"
            ));
        }
    }
    records.push(SqliteImportRecordSummary {
        source_kind: source_kind.to_string(),
        record_kind: "sidecar_store".to_string(),
        natural_key,
        record_hash,
        classification,
    });
}

fn natural_key(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(ToString::to_string)
}

fn revision_conflict_marker(value: &Value) -> bool {
    value
        .get("expected_revision")
        .and_then(Value::as_i64)
        .zip(value.get("revision").and_then(Value::as_i64))
        .is_some_and(|(expected, actual)| expected != actual)
}

fn contains_sensitive_value(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            let key_lower = key.to_ascii_lowercase().replace('-', "_");
            let allowed_numeric_token_count = matches!(
                key_lower.as_str(),
                "estimated_tokens" | "max_estimated_tokens"
            ) && matches!(value, Value::Number(_));
            (!allowed_numeric_token_count
                && SENSITIVE_KEY_PARTS
                    .iter()
                    .any(|part| key_lower.contains(part)))
                || contains_sensitive_value(value)
        }),
        Value::Array(items) => items.iter().any(contains_sensitive_value),
        Value::String(text) => {
            let lower = text.to_ascii_lowercase();
            SENSITIVE_STRING_MARKERS
                .iter()
                .any(|marker| lower.contains(marker))
        }
        _ => false,
    }
}

fn finalize_report(
    root: &Path,
    batch_status: String,
    mut source_inventory: Vec<SqliteImportSourceReport>,
    mut record_summaries: Vec<SqliteImportRecordSummary>,
    mut warnings: Vec<String>,
    mut conflicts: Vec<String>,
) -> SqliteImportDryRunReport {
    source_inventory.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.source_kind.cmp(&right.source_kind))
    });
    record_summaries.sort_by(|left, right| {
        left.source_kind
            .cmp(&right.source_kind)
            .then_with(|| left.record_kind.cmp(&right.record_kind))
            .then_with(|| left.natural_key.cmp(&right.natural_key))
    });
    warnings.sort();
    warnings.dedup();
    conflicts.sort();
    conflicts.dedup();

    let source_root_ref = root.display().to_string();
    let source_root_hash = source_root_hash(&source_inventory);
    let runtime_log_alias = runtime_log_alias_report(&source_inventory);
    let counts = counts_for(&source_inventory, &record_summaries, &warnings, &conflicts);
    SqliteImportDryRunReport {
        batch_id: format!("r3-a1-dry-run:{source_root_hash}"),
        mode: "dry_run".to_string(),
        batch_status,
        importer_version: WORKBENCH_SQLITE_IMPORTER_VERSION.to_string(),
        source_root_ref,
        source_root_hash,
        source_inventory,
        record_summaries,
        counts,
        warnings,
        conflicts,
        runtime_log_alias,
    }
}

fn counts_for(
    source_inventory: &[SqliteImportSourceReport],
    record_summaries: &[SqliteImportRecordSummary],
    warnings: &[String],
    conflicts: &[String],
) -> SqliteImportDryRunCounts {
    let files_seen = source_inventory
        .iter()
        .filter(|source| source.classification != "missing_optional")
        .count();
    let files_accepted = source_inventory
        .iter()
        .filter(|source| source.classification == "accepted")
        .count();
    let files_missing_optional = source_inventory
        .iter()
        .filter(|source| source.classification == "missing_optional")
        .count();
    let files_rejected = source_inventory
        .iter()
        .filter(|source| source.classification.starts_with("rejected"))
        .count();
    let proposed_inserts = record_summaries
        .iter()
        .filter(|record| record.classification == "accepted")
        .count();
    let skips = record_summaries
        .iter()
        .filter(|record| record.classification == "skipped_duplicate")
        .count();
    SqliteImportDryRunCounts {
        files_seen,
        files_accepted,
        files_missing_optional,
        files_rejected,
        proposed_inserts,
        proposed_updates: 0,
        skips,
        conflicts: conflicts.len()
            + record_summaries
                .iter()
                .filter(|record| record.classification == "conflict")
                .count(),
        warnings: warnings.len()
            + source_inventory
                .iter()
                .map(|source| source.warnings.len())
                .sum::<usize>(),
    }
}

fn runtime_log_alias_report(
    source_inventory: &[SqliteImportSourceReport],
) -> RuntimeLogAliasReport {
    let canonical_present = source_inventory.iter().any(|source| {
        source.source_path == CANONICAL_RUNTIME_LOG && source.classification == "accepted"
    });
    let legacy_alias_present = source_inventory.iter().any(|source| {
        source.source_path == LEGACY_RUNTIME_LOG_ALIAS && source.classification == "accepted"
    });
    RuntimeLogAliasReport {
        canonical_present,
        legacy_alias_present,
        canonical_source_kind: canonical_present.then(|| "runtime_log".to_string()),
        legacy_source_kind: legacy_alias_present.then(|| "runtime_log_legacy_alias".to_string()),
        policy:
            "runtime-logs.v1.json is canonical; runtime-log.v1.json is legacy alias/ref label only"
                .to_string(),
    }
}

fn source_root_hash(source_inventory: &[SqliteImportSourceReport]) -> String {
    let mut digest_input = String::new();
    for source in source_inventory {
        if let Some(source_hash) = &source.source_hash {
            digest_input.push_str(&source.source_path);
            digest_input.push('=');
            digest_input.push_str(source_hash);
            digest_input.push('\n');
        }
    }
    sha256_hex(digest_input.as_bytes())
}

fn source_kind_for_name(name: &str) -> &'static str {
    match name {
        PRIMARY_WORKFLOW_STATE => "workflow_state",
        "blackboard-candidates.v1.json" => "blackboard_candidate",
        "formal-memories.v1.json" => "formal_memory",
        "memory-candidates.v1.json" => "memory_candidate",
        "memory-capture-events.v1.json" => "memory_capture",
        "memory-entity-relations.v1.json" => "memory_entity_relation",
        "memory-lint.v1.json" => "memory_lint",
        "memory-patterns.v1.json" => "memory_pattern",
        "observations.v1.json" => "observation",
        "plan-authorizations.v1.json" => "plan_authorization",
        "project-proposals.v1.json" => "project_proposal",
        "real-execution-product-commands.v1.json" => "product_command",
        "global-supervisor-reviews.v1.json" => "global_supervisor_review",
        "supervisor-action-control.v1.json" => "supervisor_action_control",
        "supervisor-orchestrator.v1.json" => "supervisor_orchestrator",
        CANONICAL_RUNTIME_LOG => "runtime_log",
        LEGACY_RUNTIME_LOG_ALIAS => "runtime_log_legacy_alias",
        "session-continuations.v1.json" => "session_continuation",
        _ => "unknown_sidecar",
    }
}

pub(crate) fn canonical_json_hash(value: &Value) -> String {
    let canonical = canonical_json(value);
    sha256_hex(canonical.as_bytes())
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(items) => {
            let values = items.iter().map(canonical_json).collect::<Vec<_>>();
            format!("[{}]", values.join(","))
        }
        Value::Object(map) => {
            let mut fields = map
                .iter()
                .map(|(key, value)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap_or_else(|_| "\"\"".to_string()),
                        canonical_json(value)
                    )
                })
                .collect::<Vec<_>>();
            fields.sort();
            format!("{{{}}}", fields.join(","))
        }
    }
}

fn sorted_fixture_entries(root: &Path) -> Result<Vec<fs::DirEntry>, String> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("read fixture dir failed {}: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read fixture entry failed {}: {error}", root.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use crate::utils::fs_ops::fixture_dir;

    use super::*;

    #[test]
    fn sqlite_importer_dry_run_reports_valid_fixture_deterministically() {
        let fixture = fixture_dir("r3-a1", "valid-workflow-core");
        let first = dry_run_import_fixture_dir(&fixture).expect("first dry-run");
        let second = dry_run_import_fixture_dir(&fixture).expect("second dry-run");

        assert_eq!(first, second);
        assert_eq!(first.mode, "dry_run");
        assert_eq!(first.batch_status, "accepted");
        assert!(first.counts.proposed_inserts >= 7);
        assert!(first
            .record_summaries
            .iter()
            .any(|record| record.natural_key == "workflow:r3-a1:core"));
        assert!(first
            .source_inventory
            .iter()
            .any(|source| source.source_hash.is_some()));
    }

    #[test]
    fn sqlite_importer_dry_run_accepts_contract_fixture_matrix_domains() {
        for (name, expected_record_kind) in [
            ("valid-empty-workflow", None),
            ("valid-workflow-core", Some("workflows")),
            ("memory-adoption-chain", Some("formal_memory_record")),
            ("memory-capture-chain", Some("memory_capture_event")),
            ("proposal-authorization-chain", Some("plan_authorization")),
            ("process-fact-observation", Some("observation")),
            ("product-command-runtime-chain", Some("product_command")),
        ] {
            let report = dry_run_import_fixture_dir(&fixture_dir("r3-a1", name)).expect(name);
            assert_eq!(report.batch_status, "accepted", "{name}");
            assert!(
                report
                    .source_inventory
                    .iter()
                    .any(|source| source.classification == "missing_optional"),
                "{name} should report missing optional sidecars"
            );
            if let Some(record_kind) = expected_record_kind {
                assert!(
                    report
                        .record_summaries
                        .iter()
                        .any(|record| record.record_kind == record_kind),
                    "{name} should include record kind {record_kind}"
                );
            }
        }
    }

    #[test]
    fn sqlite_importer_dry_run_rejects_sensitive_fixture() {
        let fixture = fixture_dir("r3-a1", "forbidden-sensitive-field");
        let report = dry_run_import_fixture_dir(&fixture).expect("dry-run");
        assert_eq!(report.batch_status, "rejected_sensitive");
        assert_eq!(report.counts.proposed_inserts, 0);
        assert!(report
            .source_inventory
            .iter()
            .any(|source| source.classification == "rejected_sensitive"));
    }

    #[test]
    fn sqlite_importer_sensitive_predicate_only_allows_numeric_token_count_keys() {
        for (key, key_part) in [
            ("prompt_body", "prompt_body"),
            ("secret_x", "secret"),
            ("auth_token", "token"),
            ("user_credential", "credential"),
            ("keychain_ref", "keychain"),
            ("oauth_x", "oauth"),
            ("provider_credential", "provider_credential"),
            ("full_transcript", "full_transcript"),
            ("transcript_body", "transcript_body"),
            ("rollout_body", "rollout_body"),
        ] {
            assert!(
                contains_sensitive_value(&serde_json::json!({(key): 1})),
                "sensitive key part must remain rejected: {key_part} via {key}"
            );
        }
        assert!(contains_sensitive_value(
            &serde_json::json!({"api-token": 1})
        ));
        assert!(contains_sensitive_value(
            &serde_json::json!({"estimated_tokens": "字符串"})
        ));
        assert!(contains_sensitive_value(&serde_json::json!({"tokens": 1})));
        assert!(contains_sensitive_value(
            &serde_json::json!({"nested": {"secret_x": true}})
        ));
        assert!(contains_sensitive_value(
            &serde_json::json!([{"safe": 1}, {"oauth_x": false}])
        ));

        for value in [
            serde_json::json!({"estimated_tokens": 123}),
            serde_json::json!({"max_estimated_tokens": 456}),
            serde_json::json!({"estimated-tokens": 123}),
            serde_json::json!({"max-estimated-tokens": 456}),
        ] {
            assert!(
                !contains_sensitive_value(&value),
                "numeric token-count key should be accepted: {value}"
            );
        }
    }

    #[test]
    fn sqlite_importer_dry_run_classifies_duplicate_and_revision_conflicts() {
        let duplicate_same =
            dry_run_import_fixture_dir(&fixture_dir("r3-a1", "duplicate-same-hash"))
                .expect("dry-run");
        assert!(duplicate_same
            .record_summaries
            .iter()
            .any(|record| record.classification == "skipped_duplicate"));

        let duplicate_different =
            dry_run_import_fixture_dir(&fixture_dir("r3-a1", "duplicate-different-hash"))
                .expect("dry-run");
        assert_eq!(duplicate_different.batch_status, "conflict");
        assert!(duplicate_different.counts.conflicts > 0);

        let revision_conflict =
            dry_run_import_fixture_dir(&fixture_dir("r3-a1", "revision-conflict"))
                .expect("dry-run");
        assert_eq!(revision_conflict.batch_status, "conflict");
        assert!(revision_conflict
            .conflicts
            .iter()
            .any(|item| item.contains("revision_conflict")));
    }

    #[test]
    fn sqlite_importer_dry_run_classifies_corrupt_unknown_and_alias_sources() {
        let corrupt_primary =
            dry_run_import_fixture_dir(&fixture_dir("r3-a1", "corrupt-primary")).expect("dry-run");
        assert_eq!(corrupt_primary.batch_status, "rejected_corrupt_primary");

        let corrupt_optional =
            dry_run_import_fixture_dir(&fixture_dir("r3-a1", "corrupt-optional-sidecar"))
                .expect("dry-run");
        assert!(corrupt_optional.source_inventory.iter().any(|source| {
            source.source_path == "memory-candidates.v1.json"
                && source.classification == "rejected_corrupt"
        }));

        let unknown =
            dry_run_import_fixture_dir(&fixture_dir("r3-a1", "unknown-sidecar")).expect("dry-run");
        assert!(unknown
            .source_inventory
            .iter()
            .any(|source| source.classification == "rejected_unknown"));

        let alias = dry_run_import_fixture_dir(&fixture_dir("r3-a1", "runtime-log-alias"))
            .expect("dry-run");
        assert!(alias.runtime_log_alias.canonical_present);
        assert!(alias.runtime_log_alias.legacy_alias_present);
        assert_eq!(
            alias.runtime_log_alias.legacy_source_kind.as_deref(),
            Some("runtime_log_legacy_alias")
        );
    }

    #[test]
    fn sqlite_importer_dry_run_second_pass_marks_same_hash_as_skipped() {
        let fixture = fixture_dir("r3-a1", "valid-workflow-core");
        let first = dry_run_import_fixture_dir(&fixture).expect("first dry-run");
        let second = dry_run_import_fixture_dir_with_previous(&fixture, Some(&first))
            .expect("second dry-run");
        assert!(second.counts.skips > 0);
        assert!(second
            .record_summaries
            .iter()
            .any(|record| record.classification == "skipped_duplicate"));
    }
}
