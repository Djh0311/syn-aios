pub(crate) fn db_primary_projection_records(
    workflow_state_path: &Path,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let store = load_store_at(&sidecar_path_for_workflow_state_path(workflow_state_path)?)?;
    let sessions = store
        .sessions
        .into_iter()
        .map(|session| {
            serde_json::to_value(session)
                .map_err(|error| format!("主管编排 session 投影序列化失败：{error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let audit_events = store
        .audit_events
        .into_iter()
        .map(|audit| {
            serde_json::to_value(audit)
                .map_err(|error| format!("主管编排审计投影序列化失败：{error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((sessions, audit_events))
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    sessions: &[Value],
    audit_events: &[Value],
    replace_db_primary_leading: bool,
    write_id: &str,
) -> Result<usize, String> {
    if sessions.is_empty() && audit_events.is_empty() {
        return Ok(0);
    }
    let sidecar = sidecar_path_for_workflow_state_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| "主管编排 sidecar 没有父目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建主管编排 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let mut store = load_store_at(&sidecar)?;
    let mut changes = 0_i64;
    let mut session_ids = BTreeSet::new();
    let mut audit_ids = BTreeSet::new();

    for value in sessions {
        let session: SupervisorSession = serde_json::from_value(value.clone())
            .map_err(|error| format!("DB 主管编排 session 投影记录无法解析：{error}"))?;
        if session.run_id.trim().is_empty() || !session_ids.insert(session.run_id.clone()) {
            return Err("supervisor_orchestrator_db_primary_invalid_session_shape".to_string());
        }
        if let Some(index) = store
            .sessions
            .iter()
            .position(|existing| existing.run_id == session.run_id)
        {
            if serde_json::to_value(&store.sessions[index])
                .map_err(|error| format!("主管编排 session 投影序列化失败：{error}"))?
                != value.clone()
            {
                if !replace_db_primary_leading {
                    return Err(format!(
                        "db_json_projection_hash_mismatch:supervisor_orchestrator_sessions:{}",
                        session.run_id
                    ));
                }
                store.sessions[index] = session;
                changes += 1;
            }
        } else {
            store.sessions.push(session);
            changes += 1;
        }
    }

    for value in audit_events {
        let audit: SupervisorAuditEvent = serde_json::from_value(value.clone())
            .map_err(|error| format!("DB 主管编排审计投影记录无法解析：{error}"))?;
        if audit.event_id.trim().is_empty() || !audit_ids.insert(audit.event_id.clone()) {
            return Err("supervisor_orchestrator_db_primary_invalid_audit_shape".to_string());
        }
        if let Some(index) = store
            .audit_events
            .iter()
            .position(|existing| existing.event_id == audit.event_id)
        {
            if serde_json::to_value(&store.audit_events[index])
                .map_err(|error| format!("主管编排审计投影序列化失败：{error}"))?
                != value.clone()
            {
                if !replace_db_primary_leading {
                    return Err(format!(
                        "db_json_projection_hash_mismatch:supervisor_orchestrator_audit_events:{}",
                        audit.event_id
                    ));
                }
                store.audit_events[index] = audit;
                changes += 1;
            }
        } else {
            store.audit_events.push(audit);
            changes += 1;
        }
    }

    if changes == 0 {
        return Ok(0);
    }
    store.revision = store
        .revision
        .checked_add(changes)
        .ok_or_else(|| "主管编排 sidecar revision 已到上限".to_string())?;
    store.updated_at_ms = now_ms();
    write_store_atomic(&sidecar, &store, write_id)?;
    Ok(changes as usize)
}

#[derive(Debug)]
enum DbPrimaryStoreUpdateError {
    Store(String),
    Update(String),
    PersistDb(String),
    ProjectJson(String),
}

impl DbPrimaryStoreUpdateError {
    fn into_message(self) -> String {
        match self {
            Self::Store(message)
            | Self::Update(message)
            | Self::PersistDb(message)
            | Self::ProjectJson(message) => message,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DbPrimaryTestFailure {
    StorePrepare,
    PersistDb,
    ProjectJson,
}

#[cfg(test)]
thread_local! {
    static DB_PRIMARY_TEST_FAILURE: std::cell::Cell<Option<DbPrimaryTestFailure>> = const {
        std::cell::Cell::new(None)
    };
}

#[cfg(test)]
struct DbPrimaryTestFailureGuard;

#[cfg(test)]
impl Drop for DbPrimaryTestFailureGuard {
    fn drop(&mut self) {
        DB_PRIMARY_TEST_FAILURE.with(|failure| failure.set(None));
    }
}

#[cfg(test)]
fn force_db_primary_test_failure(failure: DbPrimaryTestFailure) -> DbPrimaryTestFailureGuard {
    DB_PRIMARY_TEST_FAILURE.with(|current| current.set(Some(failure)));
    DbPrimaryTestFailureGuard
}

#[cfg(test)]
fn db_primary_test_failure_is(failure: DbPrimaryTestFailure) -> bool {
    DB_PRIMARY_TEST_FAILURE.with(|current| current.get() == Some(failure))
}

// M5-B keeps the JSON-only writer in the parent module intact. In DB-primary mode, every
// supervisor-store mutation first records its one-session delta and newly appended audit events
// in the existing SQLite tables, then projects the complete sidecar under the same lock.
fn update_store_db_primary<R>(
    config: &McpServerConfig,
    write_id: &str,
    repository: crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
    update: impl FnOnce(&mut SupervisorStore) -> Result<R, String>,
) -> Result<R, DbPrimaryStoreUpdateError> {
    let workflow_state_path =
        workflow_state_path(config).map_err(DbPrimaryStoreUpdateError::Store)?;
    let sidecar = sidecar_path(config).map_err(DbPrimaryStoreUpdateError::Store)?;
    let parent = sidecar.parent().ok_or_else(|| {
        DbPrimaryStoreUpdateError::Store("主管编排 sidecar 没有父目录".to_string())
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        DbPrimaryStoreUpdateError::Store(format!(
            "创建主管编排 sidecar 目录失败 {}：{error}",
            parent.display()
        ))
    })?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)
        .map_err(DbPrimaryStoreUpdateError::Store)?;
    #[cfg(test)]
    if db_primary_test_failure_is(DbPrimaryTestFailure::StorePrepare) {
        return Err(DbPrimaryStoreUpdateError::Store(
            "shared_supervisor_binding_test_store_prepare_failure".to_string(),
        ));
    }
    let mut store = load_store(config).map_err(DbPrimaryStoreUpdateError::Store)?;
    let before = store.clone();
    let result = update(&mut store).map_err(DbPrimaryStoreUpdateError::Update)?;
    stamp_changed_supervisor_session(&before, &mut store, now_ms())
        .map_err(DbPrimaryStoreUpdateError::Store)?;
    store.revision += 1;
    store.updated_at_ms = now_ms();

    let changed_session =
        changed_supervisor_session(&before, &store).map_err(DbPrimaryStoreUpdateError::Store)?;
    let appended_audits =
        appended_supervisor_audits(&before, &store).map_err(DbPrimaryStoreUpdateError::Store)?;
    let session_value = changed_session
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| {
            DbPrimaryStoreUpdateError::Store(format!(
                "序列化主管编排 DB 主写 session 失败：{error}"
            ))
        })?;
    let audit_values = appended_audits
        .iter()
        .map(|audit| {
            serde_json::to_value(audit).map_err(|error| {
                DbPrimaryStoreUpdateError::Store(format!("序列化主管编排 DB 主写审计失败：{error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    #[cfg(test)]
    if db_primary_test_failure_is(DbPrimaryTestFailure::PersistDb) {
        return Err(DbPrimaryStoreUpdateError::PersistDb(
            "shared_supervisor_binding_test_persist_db_failure".to_string(),
        ));
    }
    repository
        .record_supervisor_orchestrator_delta(session_value.as_ref(), &audit_values, None)
        .map_err(DbPrimaryStoreUpdateError::PersistDb)?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        workflow_state_path,
        "supervisor_orchestrator",
        || {
            #[cfg(test)]
            if db_primary_test_failure_is(DbPrimaryTestFailure::ProjectJson) {
                return Err("shared_supervisor_binding_test_project_json_failure".to_string());
            }
            write_store_atomic(&sidecar, &store, write_id)
        },
    )
    .map_err(DbPrimaryStoreUpdateError::ProjectJson)?;
    Ok(result)
}

fn changed_supervisor_session(
    before: &SupervisorStore,
    after: &SupervisorStore,
) -> Result<Option<SupervisorSession>, String> {
    let before_by_run = supervisor_sessions_by_run(&before.sessions)?;
    let after_by_run = supervisor_sessions_by_run(&after.sessions)?;
    if before_by_run
        .keys()
        .any(|run_id| !after_by_run.contains_key(run_id))
    {
        return Err("supervisor_orchestrator_db_primary_session_deletion_unsupported".to_string());
    }
    let changed = after
        .sessions
        .iter()
        .filter(|session| {
            before_by_run
                .get(session.run_id.as_str())
                .is_none_or(|previous| *previous != *session)
        })
        .cloned()
        .collect::<Vec<_>>();
    match changed.as_slice() {
        [] => Ok(None),
        [session] => Ok(Some(session.clone())),
        _ => Err("supervisor_orchestrator_db_primary_single_session_required".to_string()),
    }
}

fn stamp_changed_supervisor_session(
    before: &SupervisorStore,
    after: &mut SupervisorStore,
    timestamp_ms: i64,
) -> Result<(), String> {
    let before_by_run = supervisor_sessions_by_run(&before.sessions)?;
    let mut changed_indices = Vec::new();
    for (index, session) in after.sessions.iter().enumerate() {
        if before_by_run
            .get(session.run_id.as_str())
            .is_none_or(|previous| **previous != *session)
        {
            changed_indices.push(index);
        }
    }
    match changed_indices.as_slice() {
        [] => Ok(()),
        [index] => {
            let session = &mut after.sessions[*index];
            let prior_updated_at_ms = before_by_run
                .get(session.run_id.as_str())
                .map(|previous| previous.updated_at_ms)
                .unwrap_or_default();
            let next_after_prior = prior_updated_at_ms.checked_add(1).ok_or_else(|| {
                "supervisor_orchestrator_db_primary_session_freshness_exhausted".to_string()
            })?;
            session.updated_at_ms = timestamp_ms.max(next_after_prior);
            Ok(())
        }
        _ => Err("supervisor_orchestrator_db_primary_single_session_required".to_string()),
    }
}

fn appended_supervisor_audits(
    before: &SupervisorStore,
    after: &SupervisorStore,
) -> Result<Vec<SupervisorAuditEvent>, String> {
    let before_by_event = supervisor_audits_by_event(&before.audit_events)?;
    let after_by_event = supervisor_audits_by_event(&after.audit_events)?;
    if before_by_event
        .keys()
        .any(|event_id| !after_by_event.contains_key(event_id))
    {
        return Err("supervisor_orchestrator_db_primary_audit_deletion_unsupported".to_string());
    }
    for (event_id, previous) in &before_by_event {
        if after_by_event.get(event_id) != Some(previous) {
            return Err("supervisor_orchestrator_db_primary_audit_mutation_unsupported".to_string());
        }
    }
    Ok(after
        .audit_events
        .iter()
        .filter(|audit| !before_by_event.contains_key(audit.event_id.as_str()))
        .cloned()
        .collect())
}

fn supervisor_sessions_by_run(
    sessions: &[SupervisorSession],
) -> Result<BTreeMap<&str, &SupervisorSession>, String> {
    let mut by_run = BTreeMap::new();
    for session in sessions {
        if session.run_id.trim().is_empty() || by_run.insert(session.run_id.as_str(), session).is_some()
        {
            return Err("supervisor_orchestrator_db_primary_invalid_session_shape".to_string());
        }
    }
    Ok(by_run)
}

fn supervisor_audits_by_event(
    audits: &[SupervisorAuditEvent],
) -> Result<BTreeMap<&str, &SupervisorAuditEvent>, String> {
    let mut by_event = BTreeMap::new();
    for audit in audits {
        if audit.event_id.trim().is_empty() || by_event.insert(audit.event_id.as_str(), audit).is_some()
        {
            return Err("supervisor_orchestrator_db_primary_invalid_audit_shape".to_string());
        }
    }
    Ok(by_event)
}
