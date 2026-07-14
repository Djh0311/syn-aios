use super::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn db_primary_projection_records(
    workflow_state_path: &Path,
) -> Result<(Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>), String> {
    let (store, _, _) = load_real_execution_product_command_store(
        workflow_state_path,
        &crate::unix_timestamp_string(),
    )?;
    Ok((
        json_values(&store.commands, "product_commands")?,
        json_values(&store.previews, "product_command_previews")?,
        json_values(&store.decisions, "product_command_decisions")?,
        json_values(&store.attempts, "product_command_attempts")?,
    ))
}

pub(crate) fn write(
    workflow_state_path: &Path,
    before: &RealExecutionProductCommandStore,
    after: &RealExecutionProductCommandStore,
    timestamp: &str,
) -> Result<(), String> {
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    else {
        return write_real_execution_product_command_store_atomic(&sidecar, after, timestamp);
    };

    let commands = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &json_values(&before.commands, "product_commands")?,
        &json_values(&after.commands, "product_commands")?,
        "product_commands",
        "product_command_id",
    )?;
    let previews = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &json_values(&before.previews, "product_command_previews")?,
        &json_values(&after.previews, "product_command_previews")?,
        "product_command_previews",
        "preview_id",
    )?;
    let decisions = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &json_values(&before.decisions, "product_command_decisions")?,
        &json_values(&after.decisions, "product_command_decisions")?,
        "product_command_decisions",
        "decision_id",
    )?;
    let attempts = crate::workbench_sqlite_repository::changed_json_records_by_field(
        &json_values(&before.attempts, "product_command_attempts")?,
        &json_values(&after.attempts, "product_command_attempts")?,
        "product_command_attempts",
        "attempt_id",
    )?;
    repository.record_product_command_delta(&commands, &previews, &decisions, &attempts, None)?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        workflow_state_path,
        "real_execution_product_command",
        || write_real_execution_product_command_store_atomic(&sidecar, after, timestamp),
    )
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    commands: &[Value],
    previews: &[Value],
    decisions: &[Value],
    attempts: &[Value],
    replace_db_primary_leading: bool,
    write_id: &str,
) -> Result<usize, String> {
    if commands.is_empty() && previews.is_empty() && decisions.is_empty() && attempts.is_empty() {
        return Ok(0);
    }
    let timestamp = crate::unix_timestamp_string();
    let (mut store, _, sidecar) =
        load_real_execution_product_command_store(workflow_state_path, &timestamp)?;
    let mut changes = 0usize;
    changes += apply_records(
        &mut store.commands,
        commands,
        |record| record.product_command_id.as_str(),
        "product_commands",
        replace_db_primary_leading,
    )?;
    changes += apply_records(
        &mut store.previews,
        previews,
        |record| record.preview_id.as_str(),
        "product_command_previews",
        replace_db_primary_leading,
    )?;
    changes += apply_records(
        &mut store.decisions,
        decisions,
        |record| record.decision_id.as_str(),
        "product_command_decisions",
        replace_db_primary_leading,
    )?;
    changes += apply_records(
        &mut store.attempts,
        attempts,
        |record| record.attempt_id.as_str(),
        "product_command_attempts",
        replace_db_primary_leading,
    )?;
    if changes == 0 {
        return Ok(0);
    }
    store.revision = store
        .revision
        .checked_add(changes as i64)
        .ok_or_else(|| "product_command_db_primary_revision_exhausted".to_string())?;
    store.updated_at = timestamp.clone();
    store.last_write_id = Some(write_id.to_string());
    validate_real_execution_product_command_store(&store)?;
    write_real_execution_product_command_store_atomic(&sidecar, &store, &timestamp)?;
    Ok(changes)
}

fn json_values<T: Serialize>(records: &[T], table: &str) -> Result<Vec<Value>, String> {
    records
        .iter()
        .map(|record| {
            serde_json::to_value(record)
                .map_err(|error| format!("{table}_db_primary_serialize_failed:{error}"))
        })
        .collect()
}

fn apply_records<T>(
    target: &mut Vec<T>,
    values: &[Value],
    key: impl Fn(&T) -> &str,
    table: &str,
    replace_db_primary_leading: bool,
) -> Result<usize, String>
where
    T: Serialize + DeserializeOwned,
{
    let mut incoming = BTreeSet::new();
    let mut changes = 0usize;
    for value in values {
        let record: T = serde_json::from_value(value.clone())
            .map_err(|error| format!("{table}_db_primary_record_parse_failed:{error}"))?;
        let record_key = key(&record).trim().to_string();
        if record_key.is_empty() || !incoming.insert(record_key.clone()) {
            return Err(format!(
                "{table}_db_primary_invalid_record_key:{record_key}"
            ));
        }
        if let Some(index) = target
            .iter()
            .position(|existing| key(existing) == record_key.as_str())
        {
            let existing = serde_json::to_value(&target[index])
                .map_err(|error| format!("{table}_db_primary_serialize_failed:{error}"))?;
            if existing != *value {
                if !replace_db_primary_leading {
                    return Err(format!(
                        "db_json_projection_hash_mismatch:{table}:{record_key}"
                    ));
                }
                target[index] = record;
                changes += 1;
            }
        } else {
            target.push(record);
            changes += 1;
        }
    }
    Ok(changes)
}
