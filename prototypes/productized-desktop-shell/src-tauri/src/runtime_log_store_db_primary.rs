use super::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn db_primary_projection_records(
    workflow_state_path: &Path,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let store = match load_store(workflow_state_path) {
        Ok(store) => store,
        Err(error) if error == "runtime_log_sidecar_missing" => {
            return Ok((Vec::new(), Vec::new()))
        }
        Err(error) => return Err(error),
    };
    Ok((
        records_as_values(&store.entries, "runtime log entries")?,
        records_as_values(&store.summaries, "runtime log summaries")?,
    ))
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    entries: &[Value],
    summaries: &[Value],
    replace_db_primary_leading: bool,
    write_id: &str,
) -> Result<usize, String> {
    if entries.is_empty() && summaries.is_empty() {
        return Ok(0);
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("runtime log sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 runtime log sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let timestamp = crate::unix_timestamp_string();
    let mut store = match load_store(workflow_state_path) {
        Ok(store) => store,
        Err(error) if error == "runtime_log_sidecar_missing" => {
            empty_store_for_replay(workflow_state_path, &sidecar, &timestamp)
        }
        Err(error) => return Err(error),
    };
    let mut changes = 0usize;
    changes += replay_records_by_key(
        &mut store.entries,
        entries,
        "runtime_log_entries",
        |value| record_key(value, "runtime_log_entries", "entry_id"),
        replace_db_primary_leading,
        true,
    )?;
    changes += replay_records_by_key(
        &mut store.summaries,
        summaries,
        "runtime_log_summaries",
        crate::workbench_sqlite_repository::runtime_log_summary_id,
        replace_db_primary_leading,
        true,
    )?;
    if changes == 0 {
        return Ok(0);
    }
    ensure_summaries_match_entries(&store.entries, &store.summaries)?;
    store.revision = store
        .revision
        .checked_add(
            i64::try_from(changes)
                .map_err(|_| "runtime_log_db_primary_replay_change_count_overflow".to_string())?,
        )
        .ok_or_else(|| "runtime_log_db_primary_replay_revision_overflow".to_string())?;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp;
    write_store_atomic(&sidecar, &store, &store.updated_at, write_id)?;
    Ok(changes)
}

pub(crate) fn write_store_with_db_primary(
    workflow_state_path: &Path,
    sidecar: &Path,
    before: &RuntimeLogStoreV1,
    after: &RuntimeLogStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    else {
        return write_store_atomic(sidecar, after, timestamp, write_id);
    };

    let before_entries = records_as_values(&before.entries, "runtime log entries")?;
    let after_entries = records_as_values(&after.entries, "runtime log entries")?;
    let before_summaries = records_as_values(&before.summaries, "runtime log summaries")?;
    let after_summaries = records_as_values(&after.summaries, "runtime log summaries")?;
    let entries = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &before_entries,
        &after_entries,
        "runtime_log_entries",
        "entry_id",
    )?;
    let summaries = crate::workbench_sqlite_repository::changed_json_records_by_key_allow_removed(
        &before_summaries,
        &after_summaries,
        "runtime_log_summaries",
        crate::workbench_sqlite_repository::runtime_log_summary_id,
    )?;
    let removed_summary_ids = crate::workbench_sqlite_repository::removed_json_record_keys_by_key(
        &before_summaries,
        &after_summaries,
        "runtime_log_summaries",
        crate::workbench_sqlite_repository::runtime_log_summary_id,
    )?;
    repository.record_runtime_log_delta(&entries, &summaries, &removed_summary_ids, None)?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        workflow_state_path,
        "runtime_log",
        || write_store_atomic(sidecar, after, timestamp, write_id),
    )
}

fn empty_store_for_replay(
    workflow_state_path: &Path,
    sidecar: &Path,
    timestamp: &str,
) -> RuntimeLogStoreV1 {
    RuntimeLogStoreV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        store_version: 1,
        storage_kind: STORAGE_KIND.to_string(),
        scope: RuntimeLogStoreScope {
            scope_kind: "workflow_state_sidecar".to_string(),
            workflow_state_path: Some(workflow_state_path.display().to_string()),
            sidecar_path: Some(sidecar.display().to_string()),
            project_roots: Vec::new(),
        },
        revision: 0,
        last_write_id: None,
        generated_by: "runtime_log_store_explicit_append_v1".to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        boundary: boundary(),
        entries: Vec::new(),
        summaries: Vec::new(),
        warnings: vec![
            "runtime_log_sidecar_explicitly_written".to_string(),
            "runtime_log_does_not_replace_audit_event".to_string(),
            "audit_event_does_not_replace_runtime_log".to_string(),
        ],
    }
}

fn records_as_values<T: Serialize>(records: &[T], label: &str) -> Result<Vec<Value>, String> {
    records
        .iter()
        .map(|record| {
            serde_json::to_value(record)
                .map_err(|error| format!("{label} DB 主写序列化失败：{error}"))
        })
        .collect()
}

fn replay_records_by_key<T, F>(
    target: &mut Vec<T>,
    values: &[Value],
    table: &str,
    key_for: F,
    replace_db_primary_leading: bool,
    mutable: bool,
) -> Result<usize, String>
where
    T: DeserializeOwned + Serialize,
    F: Fn(&Value) -> Result<String, String>,
{
    let mut current = BTreeMap::new();
    for (index, record) in target.iter().enumerate() {
        let value = serde_json::to_value(record)
            .map_err(|error| format!("{table} JSON 投影序列化失败：{error}"))?;
        let key = key_for(&value)?;
        if current.insert(key.clone(), (index, value)).is_some() {
            return Err(format!("{table}_db_primary_duplicate_json_key:{key}"));
        }
    }
    let mut incoming = BTreeMap::new();
    for value in values {
        let key = key_for(value)?;
        if incoming.insert(key.clone(), value).is_some() {
            return Err(format!("{table}_db_primary_duplicate_db_key:{key}"));
        }
    }

    let mut changes = 0usize;
    for value in values {
        let key = key_for(value)?;
        match current.get(&key) {
            Some((_, existing)) if existing == value => {}
            Some((_, _)) if !mutable || !replace_db_primary_leading => {
                return Err(format!("db_json_projection_hash_mismatch:{table}:{key}"));
            }
            Some((index, _)) => {
                target[*index] = serde_json::from_value(value.clone())
                    .map_err(|error| format!("{table} DB 投影记录无法解析：{error}"))?;
                changes += 1;
            }
            None => {
                target.push(
                    serde_json::from_value(value.clone())
                        .map_err(|error| format!("{table} DB 投影记录无法解析：{error}"))?,
                );
                changes += 1;
            }
        }
    }
    Ok(changes)
}

fn ensure_summaries_match_entries(
    entries: &[RuntimeLogEntry],
    summaries: &[RuntimeLogSummary],
) -> Result<(), String> {
    let derived = records_as_values(&summarize_entries(entries), "derived runtime log summaries")?;
    let stored = records_as_values(summaries, "runtime log summaries")?;
    let derived_by_key = values_by_key(
        &derived,
        crate::workbench_sqlite_repository::runtime_log_summary_id,
    )?;
    let stored_by_key = values_by_key(
        &stored,
        crate::workbench_sqlite_repository::runtime_log_summary_id,
    )?;
    if derived_by_key != stored_by_key {
        return Err("runtime_log_db_primary_summary_not_derived_from_entries".to_string());
    }
    Ok(())
}

fn values_by_key<F>(values: &[Value], key_for: F) -> Result<BTreeMap<String, Value>, String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    let mut records = BTreeMap::new();
    for value in values {
        let key = key_for(value)?;
        if records.insert(key.clone(), value.clone()).is_some() {
            return Err(format!(
                "runtime_log_db_primary_duplicate_summary_key:{key}"
            ));
        }
    }
    Ok(records)
}

fn record_key(value: &Value, table: &str, key_field: &str) -> Result<String, String> {
    value
        .get(key_field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{table}_db_primary_missing_{key_field}"))
}
