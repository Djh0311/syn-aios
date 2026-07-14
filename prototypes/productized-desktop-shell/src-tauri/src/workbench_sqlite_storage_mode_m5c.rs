use super::*;
use crate::workbench_sqlite_repository::runtime_log_summary_id;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub(super) struct M5cDbProjectionData {
    supervisor_reviews: Vec<DbRecord>,
    supervisor_review_audit_events: Vec<DbRecord>,
    supervisor_boundary_reviews: Vec<DbRecord>,
    supervisor_boundary_audit_events: Vec<DbRecord>,
    session_continuations: Vec<DbRecord>,
    session_continuation_attempts: Vec<DbRecord>,
    session_continuation_audit_events: Vec<DbRecord>,
    runtime_log_entries: Vec<DbRecord>,
    runtime_log_summaries: Vec<DbRecord>,
    product_commands: Vec<DbRecord>,
    product_command_previews: Vec<DbRecord>,
    product_command_decisions: Vec<DbRecord>,
    product_command_attempts: Vec<DbRecord>,
}

pub(super) fn load_db_projection_data(
    connection: &Connection,
) -> Result<M5cDbProjectionData, String> {
    Ok(M5cDbProjectionData {
        supervisor_reviews: query_records(
            connection,
            "SELECT review_id, record_hash, record_json FROM supervisor_reviews",
        )?,
        supervisor_review_audit_events: query_records(
            connection,
            "SELECT event_id, record_hash, record_json FROM supervisor_review_audit_events",
        )?,
        supervisor_boundary_reviews: query_records(
            connection,
            "SELECT review_id, record_hash, record_json FROM supervisor_boundary_reviews",
        )?,
        supervisor_boundary_audit_events: query_records(
            connection,
            "SELECT event_id, record_hash, record_json FROM supervisor_boundary_audit_events",
        )?,
        session_continuations: query_records(
            connection,
            "SELECT continuation_id, record_hash, record_json FROM session_continuations",
        )?,
        session_continuation_attempts: query_records(
            connection,
            "SELECT attempt_id, record_hash, record_json FROM session_continuation_attempts",
        )?,
        session_continuation_audit_events: query_records(
            connection,
            "SELECT event_id, record_hash, record_json FROM session_continuation_audit_events",
        )?,
        runtime_log_entries: query_records(
            connection,
            "SELECT entry_id, record_hash, record_json FROM runtime_log_entries",
        )?,
        runtime_log_summaries: query_records(
            connection,
            "SELECT summary_id, summary_hash, record_json FROM runtime_log_summaries",
        )?,
        product_commands: query_records(
            connection,
            "SELECT product_command_id, record_hash, record_json FROM product_commands",
        )?,
        product_command_previews: query_records(
            connection,
            "SELECT preview_id, record_hash, record_json FROM product_command_previews",
        )?,
        product_command_decisions: query_records(
            connection,
            "SELECT decision_id, record_hash, record_json FROM product_command_decisions",
        )?,
        product_command_attempts: query_records(
            connection,
            "SELECT attempt_id, record_hash, record_json FROM product_command_attempts",
        )?,
    })
}

pub(super) fn reconcile_tables(
    database: &M5cDbProjectionData,
    workflow_state_path: &Path,
) -> Result<Vec<DbJsonTableReconciliation>, String> {
    let (reviews, review_audits, boundary_reviews, boundary_audits) =
        crate::global_supervisor_review_store::db_primary::db_primary_projection_records(
            workflow_state_path,
        )?;
    let (continuations, continuation_attempts, continuation_audits) =
        crate::session_continuation_store::db_primary::db_primary_projection_records(
            workflow_state_path,
        )?;
    let (runtime_entries, runtime_summaries) =
        crate::runtime_log_store::db_primary::db_primary_projection_records(workflow_state_path)?;
    let (commands, previews, decisions, attempts) =
        crate::real_execution_command::db_primary::db_primary_projection_records(
            workflow_state_path,
        )?;

    Ok(vec![
        reconcile_table(
            "supervisor_reviews",
            database.supervisor_reviews.clone(),
            values_to_records(reviews, "review_id")?,
        ),
        reconcile_table(
            "supervisor_review_audit_events",
            database.supervisor_review_audit_events.clone(),
            values_to_records(review_audits, "event_id")?,
        ),
        reconcile_table(
            "supervisor_boundary_reviews",
            database.supervisor_boundary_reviews.clone(),
            values_to_records(boundary_reviews, "review_id")?,
        ),
        reconcile_table(
            "supervisor_boundary_audit_events",
            database.supervisor_boundary_audit_events.clone(),
            values_to_records(boundary_audits, "event_id")?,
        ),
        reconcile_table(
            "session_continuations",
            database.session_continuations.clone(),
            values_to_records(continuations, "continuation_id")?,
        ),
        reconcile_table(
            "session_continuation_attempts",
            database.session_continuation_attempts.clone(),
            values_to_records(continuation_attempts, "attempt_id")?,
        ),
        reconcile_table(
            "session_continuation_audit_events",
            database.session_continuation_audit_events.clone(),
            values_to_records(continuation_audits, "event_id")?,
        ),
        reconcile_table(
            "runtime_log_entries",
            database.runtime_log_entries.clone(),
            values_to_records(runtime_entries, "entry_id")?,
        ),
        reconcile_table(
            "runtime_log_summaries",
            database.runtime_log_summaries.clone(),
            values_to_records_by_key(
                runtime_summaries,
                "runtime_log_summaries",
                runtime_log_summary_id,
            )?,
        ),
        reconcile_table(
            "product_commands",
            database.product_commands.clone(),
            values_to_records(commands, "product_command_id")?,
        ),
        reconcile_table(
            "product_command_previews",
            database.product_command_previews.clone(),
            values_to_records(previews, "preview_id")?,
        ),
        reconcile_table(
            "product_command_decisions",
            database.product_command_decisions.clone(),
            values_to_records(decisions, "decision_id")?,
        ),
        reconcile_table(
            "product_command_attempts",
            database.product_command_attempts.clone(),
            values_to_records(attempts, "attempt_id")?,
        ),
    ])
}

pub(super) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    database: &M5cDbProjectionData,
    replace_db_primary_leading: bool,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<(), String> {
    crate::global_supervisor_review_store::db_primary::replay_db_primary_projection(
        workflow_state_path,
        &values(&database.supervisor_reviews),
        &values(&database.supervisor_review_audit_events),
        &values(&database.supervisor_boundary_reviews),
        &values(&database.supervisor_boundary_audit_events),
        replace_db_primary_leading,
        timestamp_ms,
        write_id,
    )?;
    crate::session_continuation_store::db_primary::replay_db_primary_projection(
        workflow_state_path,
        &values(&database.session_continuations),
        &values(&database.session_continuation_attempts),
        &values(&database.session_continuation_audit_events),
        replace_db_primary_leading,
        write_id,
    )?;
    crate::runtime_log_store::db_primary::replay_db_primary_projection(
        workflow_state_path,
        &values(&database.runtime_log_entries),
        &values(&database.runtime_log_summaries),
        replace_db_primary_leading,
        write_id,
    )?;
    crate::real_execution_command::db_primary::replay_db_primary_projection(
        workflow_state_path,
        &values(&database.product_commands),
        &values(&database.product_command_previews),
        &values(&database.product_command_decisions),
        &values(&database.product_command_attempts),
        replace_db_primary_leading,
        write_id,
    )?;
    Ok(())
}

pub(super) fn seed_db_from_json(
    repository: &WorkbenchSqliteRepository,
    workflow_state_path: &Path,
) -> Result<(), String> {
    let (reviews, review_audits, boundary_reviews, boundary_audits) =
        crate::global_supervisor_review_store::db_primary::db_primary_projection_records(
            workflow_state_path,
        )?;
    if !reviews.is_empty()
        || !review_audits.is_empty()
        || !boundary_reviews.is_empty()
        || !boundary_audits.is_empty()
    {
        repository.record_global_supervisor_review_delta(
            &reviews,
            &review_audits,
            &boundary_reviews,
            &boundary_audits,
            None,
        )?;
    }
    let (continuations, attempts, audit_events) =
        crate::session_continuation_store::db_primary::db_primary_projection_records(
            workflow_state_path,
        )?;
    if !continuations.is_empty() || !attempts.is_empty() || !audit_events.is_empty() {
        repository.record_session_continuation_delta(
            &continuations,
            &attempts,
            &audit_events,
            None,
        )?;
    }
    let (entries, summaries) =
        crate::runtime_log_store::db_primary::db_primary_projection_records(workflow_state_path)?;
    if !entries.is_empty() || !summaries.is_empty() {
        repository.record_runtime_log_delta(&entries, &summaries, &[], None)?;
    }
    let (commands, previews, decisions, attempts) =
        crate::real_execution_command::db_primary::db_primary_projection_records(
            workflow_state_path,
        )?;
    if !commands.is_empty() || !previews.is_empty() || !decisions.is_empty() || !attempts.is_empty()
    {
        repository
            .record_product_command_delta(&commands, &previews, &decisions, &attempts, None)?;
    }
    Ok(())
}

fn values_to_records_by_key(
    values: Vec<Value>,
    table: &str,
    key_for: impl Fn(&Value) -> Result<String, String>,
) -> Result<Vec<DbRecord>, String> {
    let mut records = BTreeMap::new();
    for value in values {
        let key = key_for(&value)?;
        if records
            .insert(
                key.clone(),
                DbRecord {
                    natural_key: key.clone(),
                    record_hash: record_hash(&value)?,
                    value,
                },
            )
            .is_some()
        {
            return Err(format!("db_json_reconcile_duplicate_key:{table}:{key}"));
        }
    }
    Ok(records.into_values().collect())
}

fn values(records: &[DbRecord]) -> Vec<Value> {
    records.iter().map(|record| record.value.clone()).collect()
}
