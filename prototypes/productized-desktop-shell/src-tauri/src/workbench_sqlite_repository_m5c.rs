use super::*;

impl WorkbenchSqliteRepository {
    // M5-C sidecars are deliberately separate from workflow_audit_events. Their own immutable
    // ledgers are projected into their matching repository tables in the same Immediate write.
    pub(crate) fn record_global_supervisor_review_delta(
        &self,
        reviews: &[Value],
        review_audit_events: &[Value],
        boundary_reviews: &[Value],
        boundary_audit_events: &[Value],
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        require_m5c_delta(
            "global_supervisor_review",
            &[
                reviews,
                review_audit_events,
                boundary_reviews,
                boundary_audit_events,
            ],
        )?;
        ensure_unique_keys(reviews, "supervisor_reviews", required_key("review_id"))?;
        ensure_unique_keys(
            review_audit_events,
            "supervisor_review_audit_events",
            required_key("event_id"),
        )?;
        ensure_unique_keys(
            boundary_reviews,
            "supervisor_boundary_reviews",
            required_key("review_id"),
        )?;
        ensure_unique_keys(
            boundary_audit_events,
            "supervisor_boundary_audit_events",
            required_key("event_id"),
        )?;

        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_global_supervisor_review_delta",
            failure,
            |transaction| {
                let mut rows_touched = 0;
                for review in reviews {
                    let review_id = required_text_owned(review, "review_id")?;
                    let project_id = optional_text_owned(review, "project_id");
                    let workflow_id = optional_text_owned(review, "workflow_id");
                    let (record_hash, record_json) = serialized_record_m5c(review)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO supervisor_reviews (review_id, project_id, workflow_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                             ON CONFLICT(review_id) DO UPDATE SET
                                project_id = excluded.project_id,
                                workflow_id = excluded.workflow_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                review_id,
                                project_id,
                                workflow_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for event in review_audit_events {
                    let event_id = required_text_owned(event, "event_id")?;
                    let workflow_id = optional_text_owned(event, "workflow_id");
                    let (record_hash, record_json) = serialized_record_m5c(event)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO supervisor_review_audit_events (event_id, workflow_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(event_id) DO UPDATE SET
                                workflow_id = excluded.workflow_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                event_id,
                                workflow_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for review in boundary_reviews {
                    let review_id = required_text_owned(review, "review_id")?;
                    let project_id = optional_text_owned(review, "project_id");
                    let proposal_id = optional_text_owned(review, "proposal_id");
                    let (record_hash, record_json) = serialized_record_m5c(review)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO supervisor_boundary_reviews (review_id, project_id, proposal_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                             ON CONFLICT(review_id) DO UPDATE SET
                                project_id = excluded.project_id,
                                proposal_id = excluded.proposal_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                review_id,
                                project_id,
                                proposal_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for event in boundary_audit_events {
                    let event_id = required_text_owned(event, "event_id")?;
                    let proposal_id = optional_text_owned(event, "proposal_id");
                    let (record_hash, record_json) = serialized_record_m5c(event)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO supervisor_boundary_audit_events (event_id, proposal_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(event_id) DO UPDATE SET
                                proposal_id = excluded.proposal_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                event_id,
                                proposal_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                Ok(rows_touched)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn record_session_continuation_delta(
        &self,
        continuations: &[Value],
        attempts: &[Value],
        audit_events: &[Value],
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        require_m5c_delta(
            "session_continuation",
            &[continuations, attempts, audit_events],
        )?;
        ensure_unique_keys(
            continuations,
            "session_continuations",
            required_key("continuation_id"),
        )?;
        ensure_unique_keys(
            attempts,
            "session_continuation_attempts",
            required_key("attempt_id"),
        )?;
        ensure_unique_keys(
            audit_events,
            "session_continuation_audit_events",
            required_key("event_id"),
        )?;

        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_session_continuation_delta",
            failure,
            |transaction| {
                let mut rows_touched = 0;
                for continuation in continuations {
                    let continuation_id = required_text_owned(continuation, "continuation_id")?;
                    let product_command_id = optional_text_owned(continuation, "product_command_id");
                    let (record_hash, record_json) = serialized_record_m5c(continuation)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO session_continuations (continuation_id, product_command_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(continuation_id) DO UPDATE SET
                                product_command_id = excluded.product_command_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                continuation_id,
                                product_command_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for attempt in attempts {
                    let attempt_id = required_text_owned(attempt, "attempt_id")?;
                    let continuation_id = required_text_owned(attempt, "continuation_id")?;
                    let (record_hash, record_json) = serialized_record_m5c(attempt)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO session_continuation_attempts (attempt_id, continuation_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(attempt_id) DO UPDATE SET
                                continuation_id = excluded.continuation_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                attempt_id,
                                continuation_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for event in audit_events {
                    let event_id = required_text_owned(event, "event_id")?;
                    let continuation_id = required_text_owned(event, "continuation_id")?;
                    let (record_hash, record_json) = serialized_record_m5c(event)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO session_continuation_audit_events (event_id, continuation_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(event_id) DO UPDATE SET
                                continuation_id = excluded.continuation_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                event_id,
                                continuation_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                Ok(rows_touched)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn record_runtime_log_delta(
        &self,
        entries: &[Value],
        summaries: &[Value],
        removed_summary_ids: &[String],
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        if entries.is_empty() && summaries.is_empty() && removed_summary_ids.is_empty() {
            return Err("runtime_log_delta_required".to_string());
        }
        ensure_unique_keys(entries, "runtime_log_entries", required_key("entry_id"))?;
        ensure_unique_keys(summaries, "runtime_log_summaries", runtime_log_summary_id)?;
        let mut removed = BTreeSet::new();
        for summary_id in removed_summary_ids {
            if summary_id.trim().is_empty() || !removed.insert(summary_id.as_str()) {
                return Err("runtime_log_removed_summary_id_invalid".to_string());
            }
        }
        for summary in summaries {
            let summary_id = runtime_log_summary_id(summary)?;
            if removed.contains(summary_id.as_str()) {
                return Err(format!(
                    "runtime_log_summary_upsert_delete_overlap:{summary_id}"
                ));
            }
        }

        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_runtime_log_delta",
            failure,
            |transaction| {
                let mut rows_touched = 0;
                for summary_id in removed_summary_ids {
                    rows_touched += transaction
                        .execute(
                            "DELETE FROM runtime_log_summaries WHERE summary_id = ?1",
                            [summary_id],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for entry in entries {
                    let entry_id = required_text_owned(entry, "entry_id")?;
                    let (record_hash, record_json) = serialized_record_m5c(entry)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO runtime_log_entries (entry_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4)
                             ON CONFLICT(entry_id) DO UPDATE SET
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![entry_id, REPOSITORY_SOURCE_ID, record_hash, record_json],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for summary in summaries {
                    let summary_id = runtime_log_summary_id(summary)
                        .map_err(RepositoryMutationError::Message)?;
                    let batch_id = optional_text_owned(summary, "batch_id");
                    let category = required_text_owned(summary, "category")?;
                    let status = required_text_owned(summary, "status")?;
                    let severity = required_text_owned(summary, "severity")?;
                    let (summary_hash, record_json) = serialized_record_m5c(summary)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO runtime_log_summaries (summary_id, batch_id, category, status, severity, summary_hash, source_id, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                             ON CONFLICT(summary_id) DO UPDATE SET
                                batch_id = excluded.batch_id,
                                category = excluded.category,
                                status = excluded.status,
                                severity = excluded.severity,
                                summary_hash = excluded.summary_hash,
                                source_id = excluded.source_id,
                                record_json = excluded.record_json",
                            params![
                                summary_id,
                                batch_id,
                                category,
                                status,
                                severity,
                                summary_hash,
                                REPOSITORY_SOURCE_ID,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                Ok(rows_touched)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }

    pub(crate) fn record_product_command_delta(
        &self,
        commands: &[Value],
        previews: &[Value],
        decisions: &[Value],
        attempts: &[Value],
        failure: Option<RepositoryFailurePoint>,
    ) -> Result<RepositoryReceipt, String> {
        require_m5c_delta(
            "product_command",
            &[commands, previews, decisions, attempts],
        )?;
        ensure_unique_keys(
            commands,
            "product_commands",
            required_key("product_command_id"),
        )?;
        ensure_unique_keys(
            previews,
            "product_command_previews",
            required_key("preview_id"),
        )?;
        ensure_unique_keys(
            decisions,
            "product_command_decisions",
            required_key("decision_id"),
        )?;
        ensure_unique_keys(
            attempts,
            "product_command_attempts",
            required_key("attempt_id"),
        )?;

        let (rows_touched, busy_retries) = self.with_immediate_transaction(
            "record_product_command_delta",
            failure,
            |transaction| {
                let mut rows_touched = 0;
                for command in commands {
                    let product_command_id = required_text_owned(command, "product_command_id")?;
                    let (record_hash, record_json) = serialized_record_m5c(command)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO product_commands (product_command_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4)
                             ON CONFLICT(product_command_id) DO UPDATE SET
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                product_command_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for preview in previews {
                    let preview_id = required_text_owned(preview, "preview_id")?;
                    let product_command_id = required_nested_text_owned(
                        preview,
                        "request",
                        "product_command_id",
                    )?;
                    let (record_hash, record_json) = serialized_record_m5c(preview)?;
                    let preview_hash = record_hash.clone();
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO product_command_previews (preview_id, product_command_id, preview_hash, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                             ON CONFLICT(preview_id) DO UPDATE SET
                                product_command_id = excluded.product_command_id,
                                preview_hash = excluded.preview_hash,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                preview_id,
                                product_command_id,
                                preview_hash,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for decision in decisions {
                    let decision_id = required_text_owned(decision, "decision_id")?;
                    let product_command_id = required_text_owned(decision, "product_command_id")?;
                    let (record_hash, record_json) = serialized_record_m5c(decision)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO product_command_decisions (decision_id, product_command_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(decision_id) DO UPDATE SET
                                product_command_id = excluded.product_command_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                decision_id,
                                product_command_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                for attempt in attempts {
                    let attempt_id = required_text_owned(attempt, "attempt_id")?;
                    let product_command_id = required_text_owned(attempt, "product_command_id")?;
                    let (record_hash, record_json) = serialized_record_m5c(attempt)?;
                    rows_touched += transaction
                        .execute(
                            "INSERT INTO product_command_attempts (attempt_id, product_command_id, source_id, record_hash, record_json)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(attempt_id) DO UPDATE SET
                                product_command_id = excluded.product_command_id,
                                source_id = excluded.source_id,
                                record_hash = excluded.record_hash,
                                record_json = excluded.record_json",
                            params![
                                attempt_id,
                                product_command_id,
                                REPOSITORY_SOURCE_ID,
                                record_hash,
                                record_json,
                            ],
                        )
                        .map_err(RepositoryMutationError::Sqlite)?;
                }
                Ok(rows_touched)
            },
        )?;
        Ok(RepositoryReceipt {
            rows_touched,
            busy_retries,
        })
    }
}

pub(crate) fn changed_json_records_by_field(
    before: &[Value],
    after: &[Value],
    array_name: &str,
    key_field: &str,
) -> Result<Vec<Value>, String> {
    changed_json_records_by_key(before, after, array_name, required_key(key_field))
}

fn changed_json_records_by_key<F>(
    before: &[Value],
    after: &[Value],
    array_name: &str,
    key_for: F,
) -> Result<Vec<Value>, String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    if let Some(key) = removed_json_record_keys_by_key(before, after, array_name, &key_for)?
        .into_iter()
        .next()
    {
        return Err(format!("m5c_sidecar_record_removed:{array_name}:{key}"));
    }
    changed_json_records_by_key_allow_removed(before, after, array_name, key_for)
}

pub(crate) fn appended_json_records_by_field(
    before: &[Value],
    after: &[Value],
    array_name: &str,
    key_field: &str,
) -> Result<Vec<Value>, String> {
    appended_json_records_by_key(before, after, array_name, required_key(key_field))
}

pub(crate) fn changed_json_records_by_key_allow_removed<F>(
    before: &[Value],
    after: &[Value],
    array_name: &str,
    key_for: F,
) -> Result<Vec<Value>, String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    let before_by_key = index_json_records(before, array_name, &key_for)?;
    index_json_records(after, array_name, &key_for)?;
    let mut changed = Vec::new();
    for record in after {
        let key = key_for(record)?;
        if before_by_key
            .get(&key)
            .map(|before_record| *before_record != record)
            .unwrap_or(true)
        {
            changed.push(record.clone());
        }
    }
    Ok(changed)
}

pub(crate) fn removed_json_record_keys_by_key<F>(
    before: &[Value],
    after: &[Value],
    array_name: &str,
    key_for: F,
) -> Result<Vec<String>, String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    let before_by_key = index_json_records(before, array_name, &key_for)?;
    let after_by_key = index_json_records(after, array_name, &key_for)?;
    Ok(before_by_key
        .into_keys()
        .filter(|key| !after_by_key.contains_key(key))
        .collect())
}

fn appended_json_records_by_key<F>(
    before: &[Value],
    after: &[Value],
    array_name: &str,
    key_for: F,
) -> Result<Vec<Value>, String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    let before_by_key = index_json_records(before, array_name, &key_for)?;
    let after_by_key = index_json_records(after, array_name, &key_for)?;
    for (key, before_record) in &before_by_key {
        let after_record = after_by_key
            .get(key)
            .ok_or_else(|| format!("m5c_sidecar_record_removed:{array_name}:{key}"))?;
        if *after_record != *before_record {
            return Err(format!("m5c_sidecar_append_mutated:{array_name}:{key}"));
        }
    }
    let mut appended = Vec::new();
    for record in after {
        let key = key_for(record)?;
        if !before_by_key.contains_key(&key) {
            appended.push(record.clone());
        }
    }
    Ok(appended)
}

// Runtime summaries are keyed by their natural tuple only in SQLite. The derived key never
// enters the JSON projection because RuntimeLogSummary intentionally has no summary_id field.
pub(crate) fn runtime_log_summary_id(summary: &Value) -> Result<String, String> {
    let category = required_text_json(summary, "category")?;
    let status = required_text_json(summary, "status")?;
    let severity = required_text_json(summary, "severity")?;
    let material = serde_json::to_string(&[category, status, severity])
        .map_err(|error| format!("runtime_log_summary_key_serialize_failed:{error}"))?;
    Ok(format!("runtime-log-summary:{}", sha256_hex(&material)))
}

fn require_m5c_delta(operation: &str, delta_sets: &[&[Value]]) -> Result<(), String> {
    if delta_sets.iter().all(|records| records.is_empty()) {
        return Err(format!("{operation}_delta_required"));
    }
    Ok(())
}

fn ensure_unique_keys<F>(records: &[Value], array_name: &str, key_for: F) -> Result<(), String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    index_json_records(records, array_name, &key_for).map(|_| ())
}

fn index_json_records<'a, F>(
    records: &'a [Value],
    array_name: &str,
    key_for: &F,
) -> Result<BTreeMap<String, &'a Value>, String>
where
    F: Fn(&Value) -> Result<String, String>,
{
    let mut records_by_key = BTreeMap::new();
    for record in records {
        let key = key_for(record)?;
        if records_by_key.insert(key.clone(), record).is_some() {
            return Err(format!("m5c_sidecar_duplicate_key:{array_name}:{key}"));
        }
    }
    Ok(records_by_key)
}

fn required_key<'a>(field: &'a str) -> impl Fn(&Value) -> Result<String, String> + 'a {
    move |value| required_text_json(value, field)
}

fn required_text_json(value: &Value, field: &str) -> Result<String, String> {
    required_text(value, field)
        .map(|text| text.to_string())
        .map_err(|error| error.describe())
}

fn required_text_owned(value: &Value, field: &str) -> RepositoryMutationResult<String> {
    required_text(value, field).map(ToString::to_string)
}

fn required_nested_text_owned(
    value: &Value,
    object_field: &str,
    field: &str,
) -> RepositoryMutationResult<String> {
    value
        .get(object_field)
        .and_then(Value::as_object)
        .and_then(|object| object.get(field))
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| {
            RepositoryMutationError::Message(format!(
                "repository_text_required:{object_field}.{field}"
            ))
        })
}

fn optional_text_owned(value: &Value, field: &str) -> Option<String> {
    optional_text(value, field).map(ToString::to_string)
}

fn serialized_record_m5c(value: &Value) -> RepositoryMutationResult<(String, String)> {
    serialized_record(value).map_err(RepositoryMutationError::Message)
}
