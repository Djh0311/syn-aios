use super::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn db_primary_projection_records(
    workflow_state_path: &Path,
) -> Result<(Vec<Value>, Vec<Value>, Vec<Value>), String> {
    let timestamp = crate::unix_timestamp_string();
    let store = load_store(workflow_state_path, &timestamp)?;
    Ok((
        records_as_values(&store.continuations, "session continuations")?,
        records_as_values(&store.attempts, "session continuation attempts")?,
        records_as_values(&store.audit_events, "session continuation audit events")?,
    ))
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    continuations: &[Value],
    attempts: &[Value],
    audit_events: &[Value],
    replace_db_primary_leading: bool,
    write_id: &str,
) -> Result<usize, String> {
    if continuations.is_empty() && attempts.is_empty() && audit_events.is_empty() {
        return Ok(0);
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let timestamp = crate::unix_timestamp_string();
    let mut store = load_store(workflow_state_path, &timestamp)?;
    let mut changes = 0usize;
    changes += replay_records(
        &mut store.continuations,
        continuations,
        "session_continuations",
        "continuation_id",
        replace_db_primary_leading,
        true,
    )?;
    changes += replay_records(
        &mut store.attempts,
        attempts,
        "session_continuation_attempts",
        "attempt_id",
        replace_db_primary_leading,
        true,
    )?;
    changes += replay_records(
        &mut store.audit_events,
        audit_events,
        "session_continuation_audit_events",
        "event_id",
        replace_db_primary_leading,
        false,
    )?;
    if changes == 0 {
        return Ok(0);
    }
    let project_roots = store
        .continuations
        .iter()
        .map(|continuation| continuation.project_root.clone())
        .collect::<Vec<_>>();
    for project_root in project_roots {
        remember_project_root(&mut store, &project_root);
    }
    store.revision = store
        .revision
        .checked_add(i64::try_from(changes).map_err(|_| {
            "session_continuation_db_primary_replay_change_count_overflow".to_string()
        })?)
        .ok_or_else(|| "session_continuation_db_primary_replay_revision_overflow".to_string())?;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp;
    write_store_atomic(&sidecar, &store, &store.updated_at, write_id)?;
    Ok(changes)
}

pub(crate) fn write(
    workflow_state_path: &Path,
    before: &SessionContinuationStoreV1,
    after: &SessionContinuationStoreV1,
) -> Result<(), String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    let timestamp = after.updated_at.as_str();
    let write_id = after
        .last_write_id
        .as_deref()
        .ok_or_else(|| "session_continuation_db_primary_write_id_required".to_string())?;
    let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    else {
        return write_store_atomic(&sidecar, after, timestamp, write_id);
    };

    let before_continuations = records_as_values(&before.continuations, "session continuations")?;
    let after_continuations = records_as_values(&after.continuations, "session continuations")?;
    let before_attempts = records_as_values(&before.attempts, "session continuation attempts")?;
    let after_attempts = records_as_values(&after.attempts, "session continuation attempts")?;
    let before_audits =
        records_as_values(&before.audit_events, "session continuation audit events")?;
    let after_audits = records_as_values(&after.audit_events, "session continuation audit events")?;
    let continuations = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &before_continuations,
        &after_continuations,
        "session_continuations",
        "continuation_id",
    )?;
    let attempts = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &before_attempts,
        &after_attempts,
        "session_continuation_attempts",
        "attempt_id",
    )?;
    let audits = crate::workbench_sqlite_repository::appended_json_records_by_field(
        &before_audits,
        &after_audits,
        "session_continuation_audit_events",
        "event_id",
    )?;
    repository.record_session_continuation_delta(&continuations, &attempts, &audits, None)?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        workflow_state_path,
        "session_continuation",
        || write_store_atomic(&sidecar, after, timestamp, write_id),
    )
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

fn replay_records<T>(
    target: &mut Vec<T>,
    values: &[Value],
    table: &str,
    key_field: &str,
    replace_db_primary_leading: bool,
    mutable: bool,
) -> Result<usize, String>
where
    T: DeserializeOwned + Serialize,
{
    let mut current = BTreeMap::new();
    for (index, record) in target.iter().enumerate() {
        let value = serde_json::to_value(record)
            .map_err(|error| format!("{table} JSON 投影序列化失败：{error}"))?;
        let key = record_key(&value, table, key_field)?;
        if current.insert(key.clone(), (index, value)).is_some() {
            return Err(format!("{table}_db_primary_duplicate_json_key:{key}"));
        }
    }
    let mut incoming = BTreeMap::new();
    for value in values {
        let key = record_key(value, table, key_field)?;
        if incoming.insert(key.clone(), value).is_some() {
            return Err(format!("{table}_db_primary_duplicate_db_key:{key}"));
        }
    }

    let mut changes = 0usize;
    for value in values {
        let key = record_key(value, table, key_field)?;
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

fn record_key(value: &Value, table: &str, key_field: &str) -> Result<String, String> {
    value
        .get(key_field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{table}_db_primary_missing_{key_field}"))
}
