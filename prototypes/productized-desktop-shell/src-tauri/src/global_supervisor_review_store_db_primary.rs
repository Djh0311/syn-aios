use super::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn db_primary_projection_records(
    workflow_state_path: &Path,
) -> Result<(Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>), String> {
    // Keep the parent store's intentional soft-load behavior: a damaged sidecar is treated as
    // an empty store here too, so the next successful projection preserves the original corpse
    // through the parent atomic writer's existing backup path.
    let (store, _warnings) = load_store_soft(workflow_state_path, crate::unix_timestamp_ms());
    Ok((
        records_as_values(&store.reviews, "supervisor reviews")?,
        records_as_values(&store.audit_events, "supervisor review audit events")?,
        records_as_values(&store.boundary_reviews, "supervisor boundary reviews")?,
        records_as_values(
            &store.boundary_audit_events,
            "supervisor boundary audit events",
        )?,
    ))
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    reviews: &[Value],
    review_audit_events: &[Value],
    boundary_reviews: &[Value],
    boundary_audit_events: &[Value],
    replace_db_primary_leading: bool,
    timestamp_ms: i64,
    _write_id: &str,
) -> Result<usize, String> {
    if reviews.is_empty()
        && review_audit_events.is_empty()
        && boundary_reviews.is_empty()
        && boundary_audit_events.is_empty()
    {
        return Ok(0);
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("全局主管复核 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建全局主管复核 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;

    let (mut store, _warnings) = load_store_soft(workflow_state_path, timestamp_ms);
    let mut changes = 0usize;
    changes += replay_records(
        &mut store.reviews,
        reviews,
        "supervisor_reviews",
        "review_id",
        replace_db_primary_leading,
        true,
    )?;
    changes += replay_records(
        &mut store.audit_events,
        review_audit_events,
        "supervisor_review_audit_events",
        "event_id",
        replace_db_primary_leading,
        false,
    )?;
    changes += replay_records(
        &mut store.boundary_reviews,
        boundary_reviews,
        "supervisor_boundary_reviews",
        "review_id",
        replace_db_primary_leading,
        true,
    )?;
    changes += replay_records(
        &mut store.boundary_audit_events,
        boundary_audit_events,
        "supervisor_boundary_audit_events",
        "event_id",
        replace_db_primary_leading,
        false,
    )?;
    if changes == 0 {
        return Ok(0);
    }
    store.revision = store
        .revision
        .checked_add(i64::try_from(changes).map_err(|_| {
            "global_supervisor_review_db_primary_replay_change_count_overflow".to_string()
        })?)
        .ok_or_else(|| {
            "global_supervisor_review_db_primary_replay_revision_overflow".to_string()
        })?;
    store.updated_at_ms = timestamp_ms;
    write_store_atomic(&sidecar, &store, timestamp_ms)?;
    Ok(changes)
}

pub(crate) fn write_store_with_db_primary(
    workflow_state_path: &Path,
    sidecar: &Path,
    before: &GlobalSupervisorReviewStoreV1,
    after: &GlobalSupervisorReviewStoreV1,
    timestamp_ms: i64,
) -> Result<(), String> {
    let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    else {
        return write_store_atomic(sidecar, after, timestamp_ms);
    };

    let before_reviews = records_as_values(&before.reviews, "supervisor reviews")?;
    let after_reviews = records_as_values(&after.reviews, "supervisor reviews")?;
    let before_review_audits =
        records_as_values(&before.audit_events, "supervisor review audit events")?;
    let after_review_audits =
        records_as_values(&after.audit_events, "supervisor review audit events")?;
    let before_boundary_reviews =
        records_as_values(&before.boundary_reviews, "supervisor boundary reviews")?;
    let after_boundary_reviews =
        records_as_values(&after.boundary_reviews, "supervisor boundary reviews")?;
    let before_boundary_audits = records_as_values(
        &before.boundary_audit_events,
        "supervisor boundary audit events",
    )?;
    let after_boundary_audits = records_as_values(
        &after.boundary_audit_events,
        "supervisor boundary audit events",
    )?;
    let reviews = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &before_reviews,
        &after_reviews,
        "supervisor_reviews",
        "review_id",
    )?;
    let review_audits = crate::workbench_sqlite_repository::appended_json_records_by_field(
        &before_review_audits,
        &after_review_audits,
        "supervisor_review_audit_events",
        "event_id",
    )?;
    let boundary_reviews = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &before_boundary_reviews,
        &after_boundary_reviews,
        "supervisor_boundary_reviews",
        "review_id",
    )?;
    let boundary_audits = crate::workbench_sqlite_repository::appended_json_records_by_field(
        &before_boundary_audits,
        &after_boundary_audits,
        "supervisor_boundary_audit_events",
        "event_id",
    )?;
    repository.record_global_supervisor_review_delta(
        &reviews,
        &review_audits,
        &boundary_reviews,
        &boundary_audits,
        None,
    )?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        workflow_state_path,
        "global_supervisor_review",
        || write_store_atomic(sidecar, after, timestamp_ms),
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
            Some((index, existing)) if existing == value => {}
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
