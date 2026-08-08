use super::*;
use serde_json::{json, Value};
use std::path::Path;

// M5-F1 keeps the existing eager fallback route for sidecar/direct writers, but workflow-state
// bridge callers already hold a candidate Value. They need a deferred audit route so the audit
// cannot advance that candidate's revision before its single existing CAS write.
pub(crate) enum WorkflowStateWriteRoute {
    DbPrimary(WorkbenchSqliteRepository),
    JsonOnly,
    BlockedJsonOnly(DeferredBlockedJsonOnlyDegradation),
}

pub(crate) struct DeferredBlockedJsonOnlyDegradation {
    config: DbPrimaryJsonProjectionConfig,
    reason: String,
}

pub(crate) fn primary_repository_for_write(
    workflow_state_path: &Path,
) -> Result<Option<WorkbenchSqliteRepository>, String> {
    match workflow_state_write_route(workflow_state_path)? {
        WorkflowStateWriteRoute::DbPrimary(repository) => Ok(Some(repository)),
        WorkflowStateWriteRoute::JsonOnly => Ok(None),
        WorkflowStateWriteRoute::BlockedJsonOnly(degradation) => {
            record_blocked_json_only_degradation(&degradation.config, &degradation.reason);
            Ok(None)
        }
    }
}

/// M2/T2 recovery paths deliberately do not inherit the historical
/// `BlockedJsonOnly` degradation behavior. Once a DB-primary projection gate
/// has failed, accepting a new workflow mutation through JSON would create a
/// second writer while the canonical DB is frozen. These paths must stop
/// before they create a backup, audit, receipt, dispatch, or business-state
/// transition. JSON-only installations remain supported as an explicit mode.
pub(crate) fn primary_repository_for_m2_t2_fail_closed_write(
    workflow_state_path: &Path,
    surface: &str,
) -> Result<Option<WorkbenchSqliteRepository>, String> {
    match workflow_state_write_route(workflow_state_path)? {
        WorkflowStateWriteRoute::DbPrimary(repository) => Ok(Some(repository)),
        WorkflowStateWriteRoute::JsonOnly => Ok(None),
        WorkflowStateWriteRoute::BlockedJsonOnly(degradation) => Err(format!(
            "db_primary_m2_t2_write_frozen:{surface}:{}",
            degradation.reason
        )),
    }
}

pub(crate) fn workflow_state_write_route(
    workflow_state_path: &Path,
) -> Result<WorkflowStateWriteRoute, String> {
    match storage_mode_for(workflow_state_path) {
        StorageMode::JsonOnly { .. } => Ok(WorkflowStateWriteRoute::JsonOnly),
        StorageMode::DbPrimaryJsonProjection(config) => {
            let key = workflow_state_path.to_path_buf();
            let health = health_cache().lock().expect("storage mode health lock");
            let blocked_reason = match health.get(&key) {
                Some(DbPrimaryHealth::Ready) => None,
                Some(DbPrimaryHealth::Blocked(reason)) => Some(reason.clone()),
                None => {
                    return Err(
                        "db_primary_startup_reconciliation_required: refusing DB primary write before startup reconciliation"
                            .to_string(),
                    );
                }
            };
            drop(health);
            match blocked_reason {
                Some(reason) => Ok(WorkflowStateWriteRoute::BlockedJsonOnly(
                    DeferredBlockedJsonOnlyDegradation { config, reason },
                )),
                None => WorkbenchSqliteRepository::open_confirmed(&config.repository_config())
                    .map(WorkflowStateWriteRoute::DbPrimary),
            }
        }
    }
}

impl DeferredBlockedJsonOnlyDegradation {
    pub(crate) fn write_with_degradation_audit<T>(
        self,
        value: &Value,
        write: impl FnOnce(&Value) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut recorded = degradation_audit_recorded()
            .lock()
            .expect("storage mode degradation audit lock");
        if *recorded {
            return write(value);
        }

        let timestamp = crate::unix_timestamp_string();
        let mut combined = value.clone();
        let audit_preparation: Result<(), String> = (|| {
            append_deferred_blocked_json_only_degradation_event(
                &mut combined,
                &self.config,
                &self.reason,
                &timestamp,
            )?;
            crate::backup_workflow_state_file(&self.config.workflow_state_path, &timestamp)?;
            Ok(())
        })();
        if let Err(error) = audit_preparation {
            eprintln!("storage mode=json_only degradation audit failed:{error}");
            return write(value);
        }
        let result = write(&combined)?;
        *recorded = true;
        log_blocked_json_only_degradation(&self.reason);
        Ok(result)
    }
}

fn record_blocked_json_only_degradation(config: &DbPrimaryJsonProjectionConfig, reason: &str) {
    let mut recorded = degradation_audit_recorded()
        .lock()
        .expect("storage mode degradation audit lock");
    if *recorded {
        return;
    }
    *recorded = true;

    log_blocked_json_only_degradation(reason);

    let timestamp = crate::unix_timestamp_string();
    let result = (|| {
        let mut value = crate::read_workflow_state_value(&config.workflow_state_path)?;
        append_blocked_json_only_degradation_event(&mut value, config, reason, &timestamp)?;
        crate::backup_workflow_state_file(&config.workflow_state_path, &timestamp)?;
        crate::write_validated_workflow_state(&config.workflow_state_path, &value)
    })();
    match result {
        Ok(()) => {}
        Err(error) => eprintln!("storage mode=json_only degradation audit failed:{error}"),
    }
}

fn append_deferred_blocked_json_only_degradation_event(
    value: &mut Value,
    config: &DbPrimaryJsonProjectionConfig,
    blocked_reason: &str,
    timestamp: &str,
) -> Result<(), String> {
    #[cfg(test)]
    if take_deferred_blocked_json_only_audit_append_failure_for_tests() {
        return Err("m5f1_injected_degradation_audit_append_failure".to_string());
    }
    append_blocked_json_only_degradation_event(value, config, blocked_reason, timestamp)
}

#[cfg(test)]
pub(crate) fn inject_deferred_blocked_json_only_audit_append_failure_for_tests() {
    *deferred_blocked_json_only_audit_append_failure_for_tests()
        .lock()
        .expect("deferred degradation audit failure lock") = true;
}

#[cfg(test)]
pub(crate) fn clear_deferred_blocked_json_only_audit_append_failure_for_tests() {
    *deferred_blocked_json_only_audit_append_failure_for_tests()
        .lock()
        .expect("deferred degradation audit failure lock") = false;
}

#[cfg(test)]
fn take_deferred_blocked_json_only_audit_append_failure_for_tests() -> bool {
    let mut injected = deferred_blocked_json_only_audit_append_failure_for_tests()
        .lock()
        .expect("deferred degradation audit failure lock");
    let failure = *injected;
    *injected = false;
    failure
}

#[cfg(test)]
fn deferred_blocked_json_only_audit_append_failure_for_tests() -> &'static std::sync::Mutex<bool> {
    static FAILURE: std::sync::OnceLock<std::sync::Mutex<bool>> = std::sync::OnceLock::new();
    FAILURE.get_or_init(|| std::sync::Mutex::new(false))
}

fn append_blocked_json_only_degradation_event(
    value: &mut Value,
    config: &DbPrimaryJsonProjectionConfig,
    blocked_reason: &str,
    timestamp: &str,
) -> Result<(), String> {
    let event_id = crate::workflow_audit::audit_event_identity(
        "storage-mode-degraded-json-only",
        &config.db_path_hash(),
        timestamp,
    );
    let event = json!({
        "event_id": event_id,
        "event_type": "storage_mode_degraded_json_only",
        "target_ref": config.db_path_hash(),
        "actor_ref": "workbench_storage_mode",
        "source_kind": "workspace_state",
        "permission_level": "system_runtime",
        "before_state": "db_primary_json_projection_blocked",
        "after_state": "json_only",
        "created_at": timestamp,
        "reason": format!(
            "DB 主写已冻结：{blocked_reason}；本进程已降级 json_only，数据无损；需重新 seed 恢复 DB 主写。"
        )
    });
    let audits = value
        .get_mut("audit_events")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "storage_mode_workflow_audit_array_required".to_string())?;
    audits.push(event);
    value["updated_at"] = Value::String(timestamp.to_string());
    Ok(())
}

fn log_blocked_json_only_degradation(reason: &str) {
    eprintln!(
        "storage mode=db_primary_json_projection blocked; 已降级 json_only，数据无损，需重 seed 恢复 DB 主写；reason={reason}"
    );
}
