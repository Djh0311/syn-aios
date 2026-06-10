use crate::workbench_sqlite_importer::canonical_json_hash;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteExportDryRunManifest {
    pub(crate) export_id: String,
    pub(crate) mode: String,
    pub(crate) status: String,
    pub(crate) target_root_ref: String,
    pub(crate) export_hash: String,
    pub(crate) projected_files: Vec<SqliteProjectedFile>,
    pub(crate) redaction_manifest: Vec<String>,
    pub(crate) runtime_log_alias_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SqliteProjectedFile {
    pub(crate) path: String,
    pub(crate) canonical: bool,
    pub(crate) record_count: usize,
    pub(crate) projected_hash: String,
    pub(crate) projection: Value,
}

pub(crate) fn export_temp_db_to_json_dry_run(
    db_path: &Path,
    target_root_ref: &str,
) -> Result<SqliteExportDryRunManifest, String> {
    if !is_allowed_temp_or_r3_fixture_db_path(db_path) {
        return Err(format!(
            "temp_or_fixture_path_required: refusing to export workbench sqlite outside temp or R3 fixture paths: {}",
            db_path.display()
        ));
    }
    let connection = Connection::open(db_path).map_err(|error| {
        format!(
            "open sqlite export db failed {}: {error}",
            db_path.display()
        )
    })?;

    let mut projected_files = Vec::new();
    let workflow = workflow_state_projection(&connection)?;
    projected_files.push(projected_file("workflow-state.v0.json", true, workflow));

    let formal_memories = formal_memory_projection(&connection)?;
    if array_len(&formal_memories, "records") > 0 {
        projected_files.push(projected_file(
            "formal-memories.v1.json",
            true,
            formal_memories,
        ));
    }

    let runtime_logs = runtime_log_projection(&connection)?;
    if array_len(&runtime_logs, "entries") > 0 || array_len(&runtime_logs, "summaries") > 0 {
        projected_files.push(projected_file("runtime-logs.v1.json", true, runtime_logs));
    }

    let product_commands = product_command_projection(&connection)?;
    if array_len(&product_commands, "commands") > 0 {
        projected_files.push(projected_file(
            "real-execution-product-commands.v1.json",
            true,
            product_commands,
        ));
    }

    let continuations = session_continuation_projection(&connection)?;
    if array_len(&continuations, "continuations") > 0 {
        projected_files.push(projected_file(
            "session-continuations.v1.json",
            true,
            continuations,
        ));
    }

    let redaction_manifest = vec![
        "prompt_body:omitted".to_string(),
        "full_transcript:omitted".to_string(),
        "secret_token_credential_keychain_oauth_provider_credential:omitted".to_string(),
        "rollout_body:omitted".to_string(),
    ];
    let export_input = json!({
        "target_root_ref": target_root_ref,
        "projected_files": projected_files,
        "redaction_manifest": redaction_manifest,
    });
    let export_hash = canonical_json_hash(&export_input);
    Ok(SqliteExportDryRunManifest {
        export_id: format!("r3-a2-export:{export_hash}"),
        mode: "dry_run".to_string(),
        status: "planned".to_string(),
        target_root_ref: target_root_ref.to_string(),
        export_hash,
        projected_files,
        redaction_manifest,
        runtime_log_alias_policy:
            "export emits only canonical runtime-logs.v1.json; runtime-log.v1.json remains legacy source ref"
                .to_string(),
    })
}

fn is_allowed_temp_or_r3_fixture_db_path(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    path.starts_with(std::env::temp_dir())
        || path.starts_with(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join("r3-a2"),
        )
}

fn projected_file(path: &str, canonical: bool, projection: Value) -> SqliteProjectedFile {
    let record_count = projection_record_count(&projection);
    let projected_hash = canonical_json_hash(&projection);
    SqliteProjectedFile {
        path: path.to_string(),
        canonical,
        record_count,
        projected_hash,
        projection,
    }
}

fn workflow_state_projection(connection: &Connection) -> Result<Value, String> {
    let meta = first_record_json(connection, "workflow_state_meta", "meta_json")?
        .unwrap_or_else(|| json!({}));
    Ok(json!({
        "schema_version": meta.get("schema_version").and_then(Value::as_str).unwrap_or("workflow_state_v0"),
        "workflow_version": meta.get("workflow_version").and_then(Value::as_i64).unwrap_or(1),
        "revision": meta.get("revision").and_then(Value::as_i64).unwrap_or(1),
        "projects": record_json_array(connection, "projects")?,
        "agent_adapters": record_json_array(connection, "agent_adapters")?,
        "workflows": record_json_array(connection, "workflows")?,
        "nodes": record_json_array(connection, "workflow_nodes")?,
        "edges": record_json_array(connection, "workflow_edges")?,
        "work_items": record_json_array(connection, "work_items")?,
        "artifacts": record_json_array(connection, "workflow_artifacts")?,
        "reviews": record_json_array(connection, "workflow_reviews")?,
        "audit_events": record_json_array(connection, "workflow_audit_events")?,
        "capabilities": record_json_array(connection, "capabilities")?,
        "harness_resources": record_json_array(connection, "harness_resources")?,
        "workflow_node_session_bindings": record_json_array(connection, "workflow_node_session_bindings")?,
        "workflow_node_dispatches": record_json_array(connection, "workflow_node_dispatches")?
    }))
}

fn formal_memory_projection(connection: &Connection) -> Result<Value, String> {
    Ok(json!({
        "schema_version": "memory_governance.v1",
        "revision": 1,
        "records": record_json_array(connection, "formal_memory_records")?,
        "versions": record_json_array(connection, "formal_memory_versions")?,
        "audit_events": record_json_array(connection, "formal_memory_audit_events")?
    }))
}

fn runtime_log_projection(connection: &Connection) -> Result<Value, String> {
    Ok(json!({
        "schema_version": "runtime_log_store.v1",
        "revision": 1,
        "entries": record_json_array(connection, "runtime_log_entries")?,
        "summaries": runtime_summary_json_array(connection)?
    }))
}

fn product_command_projection(connection: &Connection) -> Result<Value, String> {
    Ok(json!({
        "schema_version": "real_execution_product_commands.v1",
        "revision": 1,
        "commands": record_json_array(connection, "product_commands")?,
        "previews": record_json_array(connection, "product_command_previews")?,
        "decisions": record_json_array(connection, "product_command_decisions")?,
        "attempts": record_json_array(connection, "product_command_attempts")?
    }))
}

fn session_continuation_projection(connection: &Connection) -> Result<Value, String> {
    Ok(json!({
        "schema_version": "session_continuation_store.v1",
        "revision": 1,
        "continuations": record_json_array(connection, "session_continuations")?,
        "attempts": record_json_array(connection, "session_continuation_attempts")?,
        "audit_events": record_json_array(connection, "session_continuation_audit_events")?
    }))
}

fn record_json_array(connection: &Connection, table: &str) -> Result<Vec<Value>, String> {
    let sql = format!("SELECT record_json FROM {table} ORDER BY record_json");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("prepare export query {table} failed: {error}"))?;
    let values = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("query export table {table} failed: {error}"))?
        .map(|item| {
            item.map_err(|error| format!("read export row {table} failed: {error}"))
                .and_then(|text| {
                    serde_json::from_str(&text)
                        .map_err(|error| format!("parse export row {table} failed: {error}"))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values
        .into_iter()
        .map(redact_export_value)
        .collect::<Vec<_>>())
}

fn runtime_summary_json_array(connection: &Connection) -> Result<Vec<Value>, String> {
    let mut statement = connection
        .prepare("SELECT record_json FROM runtime_log_summaries ORDER BY record_json")
        .map_err(|error| format!("prepare runtime summaries failed: {error}"))?;
    let values = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("query runtime summaries failed: {error}"))?
        .map(|item| {
            item.map_err(|error| format!("read runtime summary failed: {error}"))
                .and_then(|text| {
                    serde_json::from_str(&text)
                        .map_err(|error| format!("parse runtime summary failed: {error}"))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values
        .into_iter()
        .map(redact_export_value)
        .collect::<Vec<_>>())
}

fn first_record_json(
    connection: &Connection,
    table: &str,
    column: &str,
) -> Result<Option<Value>, String> {
    let sql = format!("SELECT {column} FROM {table} ORDER BY {column} LIMIT 1");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("prepare first record {table} failed: {error}"))?;
    let mut rows = statement
        .query([])
        .map_err(|error| format!("query first record {table} failed: {error}"))?;
    if let Some(row) = rows
        .next()
        .map_err(|error| format!("read first record {table} failed: {error}"))?
    {
        let text: String = row
            .get(0)
            .map_err(|error| format!("read first record text {table} failed: {error}"))?;
        let value = serde_json::from_str(&text)
            .map_err(|error| format!("parse first record {table} failed: {error}"))?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn redact_export_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let filtered = map
                .into_iter()
                .filter_map(|(key, value)| {
                    if is_forbidden_export_key(&key) {
                        None
                    } else {
                        Some((key, redact_export_value(value)))
                    }
                })
                .collect::<serde_json::Map<_, _>>();
            Value::Object(filtered)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(redact_export_value).collect()),
        other => other,
    }
}

fn is_forbidden_export_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    [
        "prompt_body",
        "full_transcript",
        "transcript_body",
        "secret",
        "token",
        "credential",
        "keychain",
        "oauth",
        "provider_credential",
        "rollout_body",
    ]
    .iter()
    .any(|part| normalized.contains(part))
}

fn projection_record_count(value: &Value) -> usize {
    match value {
        Value::Object(map) => map
            .iter()
            .filter(|(key, _)| key.as_str() != "schema_version" && key.as_str() != "revision")
            .map(|(_, value)| value.as_array().map_or(0, Vec::len))
            .sum(),
        _ => 0,
    }
}

fn array_len(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

#[allow(dead_code)]
fn manifest_counts(manifest: &SqliteExportDryRunManifest) -> BTreeMap<String, usize> {
    manifest
        .projected_files
        .iter()
        .map(|file| (file.path.clone(), file.record_count))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workbench_sqlite_apply::apply_fixture_dir_to_temp_db;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn sqlite_export_dry_run_projects_workflow_runtime_and_product_command_without_writes() {
        let fixture = fixture_dir("export-dry-run-workflow-runtime");
        let db_path = temp_db("export-dry-run-workflow-runtime");
        apply_fixture_dir_to_temp_db(&fixture, &db_path, None).expect("apply fixture");

        let manifest =
            export_temp_db_to_json_dry_run(&db_path, "dry-run-target").expect("export dry-run");

        assert_eq!(manifest.mode, "dry_run");
        assert_eq!(manifest.status, "planned");
        assert!(manifest.export_hash.len() >= 64);
        for path in [
            "workflow-state.v0.json",
            "formal-memories.v1.json",
            "runtime-logs.v1.json",
            "real-execution-product-commands.v1.json",
            "session-continuations.v1.json",
        ] {
            assert!(
                manifest
                    .projected_files
                    .iter()
                    .any(|file| file.path == path),
                "missing projection {path}"
            );
        }
        assert!(!fixture.join("export-manifest.json").exists());
    }

    #[test]
    fn sqlite_export_dry_run_omits_forbidden_sensitive_fields() {
        let fixture = fixture_dir("export-dry-run-workflow-runtime");
        let db_path = temp_db("export-redaction");
        apply_fixture_dir_to_temp_db(&fixture, &db_path, None).expect("apply fixture");

        let manifest =
            export_temp_db_to_json_dry_run(&db_path, "dry-run-target").expect("export dry-run");
        let text =
            serde_json::to_string(&manifest.projected_files).expect("serialize projected files");

        assert!(!text.contains("prompt_body"));
        assert!(!text.contains("full_transcript"));
        assert!(!text.contains("rollout_body"));
        assert!(!text.contains("provider credential value"));
        assert!(manifest
            .redaction_manifest
            .iter()
            .any(|item| item.contains("prompt_body")));
    }

    #[test]
    fn sqlite_export_dry_run_uses_canonical_runtime_log_alias_policy() {
        let fixture = fixture_dir("runtime-log-alias-export-policy");
        let db_path = temp_db("runtime-log-alias-export-policy");
        apply_fixture_dir_to_temp_db(&fixture, &db_path, None).expect("apply fixture");

        let manifest =
            export_temp_db_to_json_dry_run(&db_path, "dry-run-target").expect("export dry-run");

        assert!(manifest
            .projected_files
            .iter()
            .any(|file| file.path == "runtime-logs.v1.json"));
        assert!(!manifest
            .projected_files
            .iter()
            .any(|file| file.path == "runtime-log.v1.json"));
        assert!(manifest
            .runtime_log_alias_policy
            .contains("canonical runtime-logs.v1.json"));
    }

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("r3-a2")
            .join(name)
    }

    fn temp_db(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("r3-a2-{name}-{nanos}.sqlite"))
    }
}
