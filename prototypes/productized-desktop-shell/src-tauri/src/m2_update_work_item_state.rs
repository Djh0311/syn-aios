// M2 接线：update_work_item_state 真实命令路径接上 UoW 全链
// 证据等级：TEMP-INTEGRATION（需要真实 SQLite 连接）

use crate::m2_dto::*;
use crate::m2_workflow_state::{
    UpdateWorkItemStateCommand, UpdateWorkItemStateResult, WorkItem, WorkItemStatus,
    WorkflowStateAggregate,
};
use rusqlite::{Connection, OptionalExtension, Transaction};
use serde_json::{json, Map, Value};
#[cfg(test)]
use std::path::Path;

/// Provenance is emitted only for the explicit versioned M2 port.  The
/// generated legacy command identity intentionally has no expected revision
/// and therefore must not be presented as migrated.
pub(crate) fn m2_workflow_state_sidecar_trace_context(
    command: &UpdateWorkItemStateCommand,
) -> Option<String> {
    let _ = command.expected_revision?;
    let caller_mode = if command
        .command_id
        .starts_with("workflow-state-sidecar.m2.r4:")
    {
        "R4_ACCEPTANCE"
    } else {
        "EXPLICIT_M2_REQUEST"
    };
    Some(format!(
        "repository_port_version={};schema_version={};caller_mode={caller_mode}",
        crate::workbench_sqlite_repository::M2_WORKFLOW_STATE_SIDECAR_PORT_VERSION,
        crate::workbench_sqlite_schema_m2::M2_SCHEMA_VERSION,
    ))
}

fn m2_trace_policy_ref(command: &UpdateWorkItemStateCommand, policy_ref: String) -> String {
    match m2_workflow_state_sidecar_trace_context(command) {
        Some(trace) => format!("{policy_ref};{trace}"),
        None => policy_ref,
    }
}

fn m2_trace_source_refs(command: &UpdateWorkItemStateCommand) -> String {
    let base = format!(
        "workflow_state:{}:{}",
        command.project_id, command.workflow_id
    );
    match m2_workflow_state_sidecar_trace_context(command) {
        Some(trace) => format!("{base};{trace}"),
        None => base,
    }
}

/// M2 接线：update_work_item_state 真实路径
/// 将生产命令路径接上 UoW 全链：idempotency → policy → UoW → domain state → event → audit → receipt → current snapshot
pub fn update_work_item_state_m2(
    connection: &Connection,
    command: UpdateWorkItemStateCommand,
) -> Result<UpdateWorkItemStateResult, String> {
    execute_uow_full_chain(connection, command)
}

/// M2 接线：update_work_item_state 真实路径（使用 Transaction）
/// 将生产命令路径接上 UoW 全链：idempotency → policy → UoW → domain state → event → audit → receipt → current snapshot
pub fn update_work_item_state_m2_with_transaction(
    transaction: &Transaction,
    command: UpdateWorkItemStateCommand,
) -> Result<UpdateWorkItemStateResult, String> {
    execute_uow_full_chain_with_transaction(transaction, command)
}

/// 执行 UoW 全链
fn execute_uow_full_chain(
    connection: &Connection,
    command: UpdateWorkItemStateCommand,
) -> Result<UpdateWorkItemStateResult, String> {
    // 1. Begin UoW
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("begin uow failed: {}", e))?;

    // 2. 检查幂等键
    let request_hash = update_work_item_state_request_hash(&command);

    match check_idempotency(connection, &command, &request_hash)? {
        IdempotencyResult::AlreadyProcessed(existing_receipt) => {
            // 同 command_id + idempotency_key + 相同 request_hash → 返回既有 receipt
            connection
                .execute_batch("ROLLBACK")
                .map_err(|e| format!("rollback on idempotent failed: {}", e))?;

            // 构造返回结果
            let event = WorkbenchEventEnvelopeDto {
                event_id: generate_uuid(),
                event_type: "WorkItemStateUpdateIdempotent".to_string(),
                occurred_at: generate_timestamp(),
                actor_id: command.actor_id.clone(),
                scope_ref: command.scope_ref.clone(),
                source_ref: format!(
                    "workflow_state:{}:{}",
                    command.project_id, command.workflow_id
                ),
                source_revision: existing_receipt.committed_revision.map(|r| r.to_string()),
                command_id: Some(command.command_id.clone()),
                correlation_id: Some(command.command_id.clone()),
                causation_id: Some(command.command_id.clone()),
                trace_context: None,
                schema_version: "1.0.0".to_string(),
                sensitivity: EventSensitivity::Internal,
                summary_ref: Some(format!(
                    "work_item {} state update idempotent replay",
                    command.work_item_id
                )),
                payload_ref: None,
                payload_hash: None,
                created_at: generate_timestamp(),
            };

            let audit = AuditRecordDto {
                audit_id: generate_uuid(),
                action: AuditAction::Committed,
                decision: "idempotent_replay".to_string(),
                reason_code: Some("IDEMPOTENT_REPLAY".to_string()),
                actor_id: command.actor_id.clone(),
                scope_ref: command.scope_ref.clone(),
                subject_ref: Some(format!("work_item:{}", command.work_item_id)),
                command_id: Some(command.command_id.clone()),
                correlation_id: Some(command.command_id.clone()),
                occurred_at: generate_timestamp(),
                sensitivity: AuditSensitivity::Internal,
                scrub_result: Some("no_sensitive_material".to_string()),
                source_refs: Some(format!(
                    "workflow_state:{}:{}",
                    command.project_id, command.workflow_id
                )),
                created_at: generate_timestamp(),
            };

            Ok(UpdateWorkItemStateResult {
                receipt: existing_receipt,
                event,
                audit,
                snapshot: None,
                outbox_item: None,
            })
        }
        IdempotencyResult::Conflict { existing_hash } => {
            // 同键不同 hash → 报 conflict 错误
            connection
                .execute_batch("ROLLBACK")
                .map_err(|e| format!("rollback on idempotent conflict failed: {}", e))?;
            Err(format!(
                "idempotent_conflict: command_id={}, idempotency_key={}, existing_hash={}, new_hash={}",
                command.command_id, command.idempotency_key, existing_hash, request_hash
            ))
        }
        IdempotencyResult::New => {
            // 新请求，继续执行
            // 3. 获取当前聚合状态
            let mut aggregate = get_aggregate(
                connection,
                &command.project_id,
                &command.workflow_id,
                &command.work_item_id,
            )?
            .unwrap_or_else(|| WorkflowStateAggregate {
                project_id: command.project_id.clone(),
                workflow_id: command.workflow_id.clone(),
                revision: 0,
                work_items: Vec::new(),
            });

            // 4. 验证 revision（乐观锁）
            if let Some(expected_revision) = command.expected_revision {
                if aggregate.revision != expected_revision {
                    connection
                        .execute_batch("ROLLBACK")
                        .map_err(|e| format!("rollback on revision conflict failed: {}", e))?;
                    return Err(format!(
                        "revision_conflict: expected {}, actual {}",
                        expected_revision, aggregate.revision
                    ));
                }
            }

            // 5. 查找 work item
            let work_item = aggregate
                .work_items
                .iter_mut()
                .find(|wi| wi.work_item_id == command.work_item_id);

            let work_item = match work_item {
                Some(wi) => wi,
                None => {
                    connection
                        .execute_batch("ROLLBACK")
                        .map_err(|e| format!("rollback on work item not found failed: {}", e))?;
                    return Err(format!(
                        "work_item_not_found: work_item_id={}",
                        command.work_item_id
                    ));
                }
            };

            // 6. Policy 检查（真闸：control_core 状态转换表）
            if let Some(ref new_status) = command.new_status {
                let before_state = work_item.status.to_string();
                if let Err(reason) = crate::control_core::validate_work_item_state_transition(
                    &before_state,
                    &new_status.to_string(),
                ) {
                    connection
                        .execute_batch("ROLLBACK")
                        .map_err(|e| format!("rollback on policy denied failed: {}", e))?;
                    // Policy-denied：写 scrubbed denial receipt，零 domain/event/outbox mutation
                    return create_denial_receipt(connection, &command, &reason);
                }
            }

            // 7. 应用状态变更
            let old_status = work_item.status.clone();
            let old_state_json = work_item.state_json.clone();

            if let Some(new_status) = command.new_status.clone() {
                work_item.status = new_status;
            }
            if let Some(new_state_json) = command.new_state_json.clone() {
                work_item.state_json = new_state_json;
            }

            // 7. 递增 revision
            aggregate.revision += 1;

            // 8. 创建 Command Receipt
            let receipt = create_command_receipt(
                connection,
                &command,
                &work_item.status,
                aggregate.revision,
            )?;

            // 9. 创建 Event
            let event = create_event(
                connection,
                &command,
                &old_status,
                &work_item.status,
                aggregate.revision,
            )?;

            // 10. 创建 Audit Record
            let audit = create_audit_record(connection, &command, &old_status, &work_item.status)?;

            // 11. 保存聚合
            save_aggregate(connection, &aggregate)?;

            // 12. 创建或更新 snapshot
            let snapshot = create_or_update_snapshot(connection, &command, &event, &aggregate)?;

            // JSON is an internal, rebuildable projection of this slice, not
            // an external effect.  It deliberately has no outbox item or
            // result command; the DB-primary caller validates it against the
            // authoritative snapshot after commit.

            // 14. Commit UoW
            connection
                .execute_batch("COMMIT")
                .map_err(|e| format!("commit uow failed: {}", e))?;

            Ok(UpdateWorkItemStateResult {
                receipt,
                event,
                audit,
                snapshot: Some(snapshot),
                outbox_item: None,
            })
        }
    }
}

/// 幂等检查结果
enum IdempotencyResult {
    AlreadyProcessed(CommandReceiptDto),
    Conflict { existing_hash: String },
    New,
}

/// 检查幂等键
fn check_idempotency(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    request_hash: &str,
) -> Result<IdempotencyResult, String> {
    let mut stmt = connection
        .prepare(
            "SELECT receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref,
                current_object_ref, policy_decision_ref, status, correlation_id, accepted_at,
                result_ref, result_hash, committed_revision, error_code, created_at
         FROM command_receipts
         WHERE command_id = ?1 AND idempotency_key = ?2",
        )
        .map_err(|e| format!("prepare idempotency check failed: {}", e))?;

    let result = stmt
        .query_row(
            rusqlite::params![command.command_id, command.idempotency_key],
            |row| {
                Ok(CommandReceiptDto {
                    receipt_id: row.get(0)?,
                    command_id: row.get(1)?,
                    idempotency_key: row.get(2)?,
                    request_hash: row.get(3)?,
                    actor_id: row.get(4)?,
                    scope_ref: row.get(5)?,
                    current_object_ref: row.get(6)?,
                    policy_decision_ref: row.get(7)?,
                    status: match row.get::<_, String>(8)?.as_str() {
                        "DENIED" => CommandReceiptStatus::Denied,
                        "NEEDS_CONFIRMATION" => CommandReceiptStatus::NeedsConfirmation,
                        "COMMITTED" => CommandReceiptStatus::Committed,
                        "EXTERNAL_PENDING" => CommandReceiptStatus::ExternalPending,
                        "EXTERNAL_RESULT" => CommandReceiptStatus::ExternalResult,
                        "PROJECTION_DEGRADED" => CommandReceiptStatus::ProjectionDegraded,
                        "FAILED" => CommandReceiptStatus::Failed,
                        _ => CommandReceiptStatus::Failed,
                    },
                    correlation_id: row.get(9)?,
                    accepted_at: row.get(10)?,
                    result_ref: row.get(11)?,
                    result_hash: row.get(12)?,
                    committed_revision: row.get(13)?,
                    error_code: row.get(14)?,
                    created_at: row.get(15)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("query idempotency failed: {}", e))?;

    match result {
        Some(existing_receipt) => {
            // 检查 request_hash 是否匹配
            if existing_receipt.request_hash == request_hash {
                // 同 command_id + idempotency_key + 相同 request_hash → 返回既有 receipt
                Ok(IdempotencyResult::AlreadyProcessed(existing_receipt))
            } else {
                // 同键不同 hash → 报 conflict 错误
                Ok(IdempotencyResult::Conflict {
                    existing_hash: existing_receipt.request_hash,
                })
            }
        }
        None => {
            // 新请求
            Ok(IdempotencyResult::New)
        }
    }
}

/// Load the single M2 aggregate together with its authoritative sidecar-meta
/// binding.  A work item is not allowed to borrow a revision by workflow id:
/// `workflow_state_meta.workspace_id` is a different key space.  The original
/// imported source binding is therefore the narrow, inspectable association.
fn get_aggregate(
    connection: &Connection,
    project_id: &str,
    workflow_id: &str,
    work_item_id: &str,
) -> Result<Option<WorkflowStateAggregate>, String> {
    let row: Option<(String, Option<String>, String, String)> = connection
        .query_row(
            "SELECT work_item_id, node_id, source_id, record_json
             FROM work_items WHERE work_item_id = ?1 AND workflow_id = ?2",
            rusqlite::params![work_item_id, workflow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|e| format!("get_aggregate: query failed: {}", e))?;

    let (work_items, source_id): (Vec<WorkItem>, String) = match row {
        Some((wi_id, node_id, source_id, record_json)) => {
            let value: Value =
                serde_json::from_str(&record_json).unwrap_or_else(|_| Value::Object(Map::new()));
            let status_str = value
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("draft");
            (
                vec![WorkItem {
                    work_item_id: wi_id,
                    node_id: node_id.unwrap_or_default(),
                    status: WorkItemStatus::from_str(status_str),
                    state_json: record_json,
                }],
                source_id,
            )
        }
        None => return Ok(None),
    };

    let meta: Option<(String, String, Option<i64>)> = connection
        .query_row(
            "SELECT workspace_id, source_root_hash, revision
             FROM workflow_state_meta WHERE source_id = ?1",
            [source_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| format!("m2_workflow_state_meta_binding_query_failed:{error}"))?;
    let (workspace_id, source_root_hash, revision) = meta.ok_or_else(|| {
        format!(
            "m2_workflow_state_meta_binding_missing:work_item_id={work_item_id}:source_id={source_id}"
        )
    })?;
    if workspace_id.trim().is_empty() || source_root_hash.trim().is_empty() {
        return Err("m2_workflow_state_meta_binding_invalid".to_string());
    }

    Ok(Some(WorkflowStateAggregate {
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        revision: revision.unwrap_or(0),
        work_items,
    }))
}

/// Persist the M2 aggregate revision with a compare-and-swap on the exact
/// workspace/root/source binding.  The caller's surrounding BEGIN IMMEDIATE
/// makes the receipt/event/audit/snapshot/outbox and this revision one UoW;
/// this CAS remains a second fail-closed guard against a stale binding.
fn save_aggregate(
    connection: &Connection,
    aggregate: &WorkflowStateAggregate,
) -> Result<(), String> {
    let work_item = aggregate
        .work_items
        .first()
        .ok_or_else(|| "m2_workflow_state_meta_binding_missing_work_item".to_string())?;
    let binding: Option<(String, String, String, Option<i64>, String)> = connection
        .query_row(
            "SELECT meta.workspace_id, meta.source_root_hash, meta.source_id, meta.revision, meta.meta_json
             FROM work_items AS item
             JOIN workflow_state_meta AS meta ON meta.source_id = item.source_id
             WHERE item.work_item_id = ?1 AND item.workflow_id = ?2",
            rusqlite::params![work_item.work_item_id, aggregate.workflow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .optional()
        .map_err(|error| format!("m2_workflow_state_meta_binding_query_failed:{error}"))?;
    let (workspace_id, source_root_hash, source_id, prior_revision, raw_meta) = binding
        .ok_or_else(|| {
            format!(
                "m2_workflow_state_meta_binding_missing:work_item_id={}",
                work_item.work_item_id
            )
        })?;
    let prior_revision = prior_revision.unwrap_or(0);
    let expected_prior_revision = aggregate
        .revision
        .checked_sub(1)
        .ok_or_else(|| "m2_workflow_state_revision_underflow".to_string())?;
    if prior_revision != expected_prior_revision {
        return Err(format!(
            "m2_workflow_state_revision_cas_precondition_failed:expected={expected_prior_revision},actual={prior_revision}"
        ));
    }
    let mut meta_json: Value = serde_json::from_str(&raw_meta)
        .map_err(|error| format!("m2_workflow_state_meta_json_invalid:{error}"))?;
    let meta_object = meta_json
        .as_object_mut()
        .ok_or_else(|| "m2_workflow_state_meta_json_not_object".to_string())?;
    meta_object.insert(
        "revision".to_string(),
        Value::Number(aggregate.revision.into()),
    );
    let next_meta_json = serde_json::to_string(&meta_json)
        .map_err(|error| format!("m2_workflow_state_meta_json_serialize_failed:{error}"))?;
    let rows = connection
        .execute(
            "UPDATE workflow_state_meta
             SET revision = ?1, meta_json = ?2
             WHERE workspace_id = ?3 AND source_root_hash = ?4 AND source_id = ?5
               AND COALESCE(revision, 0) = ?6",
            rusqlite::params![
                aggregate.revision,
                next_meta_json,
                workspace_id,
                source_root_hash,
                source_id,
                prior_revision,
            ],
        )
        .map_err(|error| format!("m2_workflow_state_revision_cas_failed:{error}"))?;
    if rows != 1 {
        return Err("m2_workflow_state_revision_cas_conflict".to_string());
    }
    Ok(())
}

/// 创建 Command Receipt
fn create_command_receipt(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    new_status: &WorkItemStatus,
    revision: i64,
) -> Result<CommandReceiptDto, String> {
    ensure_m2_command_and_correlation_registry(connection, command)?;
    let receipt = CommandReceiptDto {
        receipt_id: generate_uuid(),
        command_id: command.command_id.clone(),
        idempotency_key: command.idempotency_key.clone(),
        request_hash: update_work_item_state_request_hash(command),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        current_object_ref: Some(format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        )),
        policy_decision_ref: m2_trace_policy_ref(command, "policy_gateway:allowed".to_string()),
        status: CommandReceiptStatus::Committed,
        correlation_id: Some(command.command_id.clone()),
        accepted_at: generate_timestamp(),
        result_ref: Some(format!("work_item:{}:{}", command.work_item_id, revision)),
        result_hash: Some(sha256_hex(&new_status.to_string())),
        committed_revision: Some(revision),
        error_code: None,
        created_at: generate_timestamp(),
    };

    // 插入数据库
    connection.execute(
        "INSERT INTO command_receipts (receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref, current_object_ref, policy_decision_ref, status, correlation_id, accepted_at, result_ref, result_hash, committed_revision, error_code, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        rusqlite::params![
            receipt.receipt_id,
            receipt.command_id,
            receipt.idempotency_key,
            receipt.request_hash,
            receipt.actor_id,
            receipt.scope_ref,
            receipt.current_object_ref,
            receipt.policy_decision_ref,
            receipt.status.to_string(),
            receipt.correlation_id,
            receipt.accepted_at,
            receipt.result_ref,
            receipt.result_hash,
            receipt.committed_revision,
            receipt.error_code,
            receipt.created_at,
        ],
    ).map_err(|e| format!("insert receipt failed: {}", e))?;

    Ok(receipt)
}

/// 创建 Event
fn create_event(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    old_status: &WorkItemStatus,
    new_status: &WorkItemStatus,
    revision: i64,
) -> Result<WorkbenchEventEnvelopeDto, String> {
    let event = WorkbenchEventEnvelopeDto {
        event_id: generate_uuid(),
        event_type: "WorkItemStateUpdated".to_string(),
        occurred_at: generate_timestamp(),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        source_ref: format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        ),
        source_revision: Some(revision.to_string()),
        command_id: Some(command.command_id.clone()),
        correlation_id: Some(command.command_id.clone()),
        causation_id: Some(command.command_id.clone()),
        trace_context: m2_workflow_state_sidecar_trace_context(command),
        schema_version: "1.0.0".to_string(),
        sensitivity: EventSensitivity::Internal,
        summary_ref: Some(format!(
            "work_item {} status {} -> {}",
            command.work_item_id, old_status, new_status
        )),
        payload_ref: Some(format!("work_item:{}:{}", command.work_item_id, revision)),
        payload_hash: Some(sha256_hex(&new_status.to_string())),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    connection.execute(
        "INSERT INTO events (event_id, event_type, occurred_at, actor_id, scope_ref, source_ref, source_revision, command_id, correlation_id, causation_id, trace_context, schema_version, sensitivity, summary_ref, payload_ref, payload_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        rusqlite::params![
            event.event_id,
            event.event_type,
            event.occurred_at,
            event.actor_id,
            event.scope_ref,
            event.source_ref,
            event.source_revision,
            event.command_id,
            event.correlation_id,
            event.causation_id,
            event.trace_context,
            event.schema_version,
            event.sensitivity.to_string(),
            event.summary_ref,
            event.payload_ref,
            event.payload_hash,
            event.created_at,
        ],
    ).map_err(|e| format!("insert event failed: {}", e))?;

    Ok(event)
}

/// 创建 Audit Record
fn create_audit_record(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    old_status: &WorkItemStatus,
    new_status: &WorkItemStatus,
) -> Result<AuditRecordDto, String> {
    let audit = AuditRecordDto {
        audit_id: generate_uuid(),
        action: AuditAction::Committed,
        decision: format!(
            "work_item {} updated: status {} -> {}",
            command.work_item_id, old_status, new_status
        ),
        reason_code: Some("policy_allowed".to_string()),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        subject_ref: Some(format!("work_item:{}", command.work_item_id)),
        command_id: Some(command.command_id.clone()),
        correlation_id: Some(command.command_id.clone()),
        occurred_at: generate_timestamp(),
        sensitivity: AuditSensitivity::Internal,
        scrub_result: Some("no_sensitive_material".to_string()),
        source_refs: Some(m2_trace_source_refs(command)),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    connection.execute(
        "INSERT INTO audit_records (audit_id, action, decision, reason_code, actor_id, scope_ref, subject_ref, command_id, correlation_id, occurred_at, sensitivity, scrub_result, source_refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            audit.audit_id,
            audit.action.to_string(),
            audit.decision,
            audit.reason_code,
            audit.actor_id,
            audit.scope_ref,
            audit.subject_ref,
            audit.command_id,
            audit.correlation_id,
            audit.occurred_at,
            audit.sensitivity.to_string(),
            audit.scrub_result,
            audit.source_refs,
            audit.created_at,
        ],
    ).map_err(|e| format!("insert audit failed: {}", e))?;

    Ok(audit)
}

/// 创建或更新 Snapshot
fn create_or_update_snapshot(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    event: &WorkbenchEventEnvelopeDto,
    aggregate: &WorkflowStateAggregate,
) -> Result<CurrentSnapshotDto, String> {
    ensure_m2_projector_registry(connection, "workflow_projector", "v1")?;
    let snapshot = CurrentSnapshotDto {
        object_ref: format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        ),
        object_revision: aggregate.revision,
        source_watermark: event.event_id.clone(),
        snapshot_hash: canonical_workflow_state_sidecar_snapshot_hash_for_aggregate(
            command, aggregate,
        ),
        projector_id: "workflow_projector".to_string(),
        built_at: generate_timestamp(),
    };

    // 插入或更新数据库
    connection.execute(
        "INSERT OR REPLACE INTO current_snapshots (object_ref, object_revision, source_watermark, snapshot_hash, projector_id, built_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            snapshot.object_ref,
            snapshot.object_revision,
            snapshot.source_watermark,
            snapshot.snapshot_hash,
            snapshot.projector_id,
            snapshot.built_at,
        ],
    ).map_err(|e| format!("upsert snapshot failed: {}", e))?;

    Ok(snapshot)
}

/// 创建 Denial Receipt（policy-denied 走独立 append-only scrubbed denial receipt，零 domain/event/outbox mutation）
fn create_denial_receipt(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    reason: &str,
) -> Result<UpdateWorkItemStateResult, String> {
    // Begin UoW
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("begin uow failed: {}", e))?;
    ensure_m2_command_and_correlation_registry(connection, command)?;

    // 创建 denial receipt
    let receipt = CommandReceiptDto {
        receipt_id: generate_uuid(),
        command_id: command.command_id.clone(),
        idempotency_key: command.idempotency_key.clone(),
        request_hash: update_work_item_state_request_hash(command),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        current_object_ref: Some(format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        )),
        policy_decision_ref: m2_trace_policy_ref(
            command,
            format!("policy_gateway:denied:{}", reason),
        ),
        status: CommandReceiptStatus::Denied,
        correlation_id: Some(command.command_id.clone()),
        accepted_at: generate_timestamp(),
        result_ref: None,
        result_hash: None,
        committed_revision: None,
        error_code: Some("POLICY_DENIED".to_string()),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    connection.execute(
        "INSERT INTO command_receipts (receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref, current_object_ref, policy_decision_ref, status, correlation_id, accepted_at, result_ref, result_hash, committed_revision, error_code, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        rusqlite::params![
            receipt.receipt_id,
            receipt.command_id,
            receipt.idempotency_key,
            receipt.request_hash,
            receipt.actor_id,
            receipt.scope_ref,
            receipt.current_object_ref,
            receipt.policy_decision_ref,
            receipt.status.to_string(),
            receipt.correlation_id,
            receipt.accepted_at,
            receipt.result_ref,
            receipt.result_hash,
            receipt.committed_revision,
            receipt.error_code,
            receipt.created_at,
        ],
    ).map_err(|e| format!("insert denial receipt failed: {}", e))?;

    // 创建 denial audit record（append-only，零 domain/event/outbox mutation）
    let audit = AuditRecordDto {
        audit_id: generate_uuid(),
        action: AuditAction::Denied,
        decision: format!("policy_denied: {}", reason),
        reason_code: Some("POLICY_DENIED".to_string()),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        subject_ref: Some(format!("work_item:{}", command.work_item_id)),
        command_id: Some(command.command_id.clone()),
        correlation_id: Some(command.command_id.clone()),
        occurred_at: generate_timestamp(),
        sensitivity: AuditSensitivity::Internal,
        scrub_result: Some("no_sensitive_material".to_string()),
        source_refs: Some(m2_trace_source_refs(command)),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    connection.execute(
        "INSERT INTO audit_records (audit_id, action, decision, reason_code, actor_id, scope_ref, subject_ref, command_id, correlation_id, occurred_at, sensitivity, scrub_result, source_refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            audit.audit_id,
            audit.action.to_string(),
            audit.decision,
            audit.reason_code,
            audit.actor_id,
            audit.scope_ref,
            audit.subject_ref,
            audit.command_id,
            audit.correlation_id,
            audit.occurred_at,
            audit.sensitivity.to_string(),
            audit.scrub_result,
            audit.source_refs,
            audit.created_at,
        ],
    ).map_err(|e| format!("insert denial audit failed: {}", e))?;

    // Commit UoW
    connection
        .execute_batch("COMMIT")
        .map_err(|e| format!("commit uow failed: {}", e))?;

    // 返回结果（零 domain/event/outbox mutation）
    Ok(UpdateWorkItemStateResult {
        receipt,
        event: WorkbenchEventEnvelopeDto {
            event_id: generate_uuid(),
            event_type: "WorkItemStateUpdateDenied".to_string(),
            occurred_at: generate_timestamp(),
            actor_id: command.actor_id.clone(),
            scope_ref: command.scope_ref.clone(),
            source_ref: format!(
                "workflow_state:{}:{}",
                command.project_id, command.workflow_id
            ),
            source_revision: None,
            command_id: Some(command.command_id.clone()),
            correlation_id: Some(command.command_id.clone()),
            causation_id: Some(command.command_id.clone()),
            trace_context: None,
            schema_version: "1.0.0".to_string(),
            sensitivity: EventSensitivity::Internal,
            summary_ref: Some(format!(
                "work_item {} state update denied: {}",
                command.work_item_id, reason
            )),
            payload_ref: None,
            payload_hash: None,
            created_at: generate_timestamp(),
        },
        audit,
        snapshot: None,
        outbox_item: None,
    })
}

/// Register only the stable identity pair owned by the versioned M2 command
/// before it can become a foreign-key target.  These are the exact frozen
/// `commands` / `correlation_chains` targets; it does not convert anonymous
/// legacy callers into M2 callers.
fn ensure_m2_command_and_correlation_registry(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT OR IGNORE INTO commands (command_id, registered_at) VALUES (?1, ?2)",
            rusqlite::params![command.command_id, generate_timestamp()],
        )
        .map_err(|error| format!("m2_commands_insert_failed:{error}"))?;
    connection
        .execute(
            "INSERT OR IGNORE INTO correlation_chains (correlation_id, registered_at) VALUES (?1, ?2)",
            rusqlite::params![command.command_id, generate_timestamp()],
        )
        .map_err(|error| format!("m2_correlation_chains_insert_failed:{error}"))?;
    Ok(())
}

fn ensure_m2_projector_registry(
    connection: &Connection,
    projector_id: &str,
    projector_version: &str,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO projectors (projector_id, projector_version, registered_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(projector_id) DO UPDATE SET projector_version = excluded.projector_version",
            rusqlite::params![projector_id, projector_version, generate_timestamp()],
        )
        .map_err(|error| format!("m2_projectors_insert_failed:{error}"))?;
    Ok(())
}

/// 生成 UUIDv7（UTC epoch + 随机尾部）
fn generate_uuid() -> String {
    crate::m2_clock::uuid_v7()
}

/// 生成 ISO 8601 时间戳
fn generate_timestamp() -> String {
    crate::m2_clock::utc_now_rfc3339()
}

/// SHA-256 哈希
fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Canonical hash for one logical M2 workflow-state command.
///
/// The prior four-field formula aliased denied-then-legal and state-cycle
/// commands.  Keep the complete identity/policy/aggregate precondition in the
/// hash so an existing key can only replay the exact command it originally
/// represented. `new_state_json` is represented by a canonical JSON hash when
/// possible, never by caller formatting.
pub(crate) fn update_work_item_state_request_hash(command: &UpdateWorkItemStateCommand) -> String {
    let new_state_json_hash = command
        .new_state_json
        .as_deref()
        .map(canonical_command_state_hash);
    let canonical_payload = json!({
        "schema_version": "workflow-state-sidecar.command.v1",
        "command_id": command.command_id,
        "idempotency_key": command.idempotency_key,
        "actor_id": command.actor_id,
        "scope_ref": command.scope_ref,
        "project_id": command.project_id,
        "workflow_id": command.workflow_id,
        "work_item_id": command.work_item_id,
        "expected_revision": command.expected_revision,
        "new_status": command.new_status.as_ref().map(ToString::to_string),
        "new_state_json_hash": new_state_json_hash,
    });
    let canonical = serde_json::to_string(&canonical_payload)
        .expect("M2 command payload serializes from fixed scalar fields");
    sha256_hex(&canonical)
}

/// The M2 reference slice hashes the canonical *state snapshot*, never event
/// metadata.  This makes semantically equivalent JSON stable across key order
/// and lets the DB-primary projector compare the same snapshot after rebuild.
pub(crate) fn canonical_workflow_state_sidecar_snapshot_hash(
    project_id: &str,
    workflow_id: &str,
    revision: i64,
    work_item: &Value,
    node: &Value,
) -> String {
    crate::workbench_sqlite_importer::canonical_json_hash(&json!({
        "schema_version": "workflow-state-sidecar.snapshot.v1",
        "project_id": project_id,
        "workflow_id": workflow_id,
        "revision": revision,
        "work_item": work_item,
        "node": node,
    }))
}

fn canonical_workflow_state_sidecar_snapshot_hash_for_aggregate(
    command: &UpdateWorkItemStateCommand,
    aggregate: &WorkflowStateAggregate,
) -> String {
    let work_item = aggregate.work_items.first().map_or_else(
        || json!({"work_item_id": command.work_item_id, "state": "missing"}),
        |item| {
            let state_json = serde_json::from_str::<Value>(&item.state_json)
                .unwrap_or_else(|_| Value::String(item.state_json.clone()));
            json!({
                "work_item_id": item.work_item_id,
                "node_id": item.node_id,
                "state": item.status.to_string(),
                "state_json": state_json,
            })
        },
    );
    let node = json!({
        "node_id": work_item.get("node_id").cloned().unwrap_or(Value::Null),
        "state": work_item.get("state").cloned().unwrap_or(Value::Null),
    });
    canonical_workflow_state_sidecar_snapshot_hash(
        &command.project_id,
        &command.workflow_id,
        aggregate.revision,
        &work_item,
        &node,
    )
}

fn canonical_command_state_hash(raw: &str) -> String {
    match serde_json::from_str::<Value>(raw) {
        Ok(value) => crate::workbench_sqlite_importer::canonical_json_hash(&value),
        Err(_) => sha256_hex(raw),
    }
}

/// 执行 UoW 全链（使用 Transaction）
fn execute_uow_full_chain_with_transaction(
    transaction: &Transaction,
    command: UpdateWorkItemStateCommand,
) -> Result<UpdateWorkItemStateResult, String> {
    // 1. 检查幂等键
    let request_hash = update_work_item_state_request_hash(&command);

    match check_idempotency(transaction, &command, &request_hash)? {
        IdempotencyResult::AlreadyProcessed(existing_receipt) => {
            // 同 command_id + idempotency_key + 相同 request_hash → 返回既有 receipt
            // 构造返回结果
            let event = WorkbenchEventEnvelopeDto {
                event_id: generate_uuid(),
                event_type: "WorkItemStateUpdateIdempotent".to_string(),
                occurred_at: generate_timestamp(),
                actor_id: command.actor_id.clone(),
                scope_ref: command.scope_ref.clone(),
                source_ref: format!(
                    "workflow_state:{}:{}",
                    command.project_id, command.workflow_id
                ),
                source_revision: existing_receipt.committed_revision.map(|r| r.to_string()),
                command_id: Some(command.command_id.clone()),
                correlation_id: Some(command.command_id.clone()),
                causation_id: Some(command.command_id.clone()),
                trace_context: None,
                schema_version: "1.0.0".to_string(),
                sensitivity: EventSensitivity::Internal,
                summary_ref: Some(format!(
                    "work_item {} state update idempotent replay",
                    command.work_item_id
                )),
                payload_ref: None,
                payload_hash: None,
                created_at: generate_timestamp(),
            };

            let audit = AuditRecordDto {
                audit_id: generate_uuid(),
                action: AuditAction::Committed,
                decision: "idempotent_replay".to_string(),
                reason_code: Some("IDEMPOTENT_REPLAY".to_string()),
                actor_id: command.actor_id.clone(),
                scope_ref: command.scope_ref.clone(),
                subject_ref: Some(format!("work_item:{}", command.work_item_id)),
                command_id: Some(command.command_id.clone()),
                correlation_id: Some(command.command_id.clone()),
                occurred_at: generate_timestamp(),
                sensitivity: AuditSensitivity::Internal,
                scrub_result: Some("no_sensitive_material".to_string()),
                source_refs: Some(format!(
                    "workflow_state:{}:{}",
                    command.project_id, command.workflow_id
                )),
                created_at: generate_timestamp(),
            };

            Ok(UpdateWorkItemStateResult {
                receipt: existing_receipt,
                event,
                audit,
                snapshot: None,
                outbox_item: None,
            })
        }
        IdempotencyResult::Conflict { existing_hash } => {
            // 同键不同 hash → 报 conflict 错误
            Err(format!(
                "idempotent_conflict: command_id={}, idempotency_key={}, existing_hash={}, new_hash={}",
                command.command_id, command.idempotency_key, existing_hash, request_hash
            ))
        }
        IdempotencyResult::New => {
            // 新请求，继续执行
            // 2. 获取当前聚合状态
            let mut aggregate = get_aggregate(
                transaction,
                &command.project_id,
                &command.workflow_id,
                &command.work_item_id,
            )?
            .unwrap_or_else(|| WorkflowStateAggregate {
                project_id: command.project_id.clone(),
                workflow_id: command.workflow_id.clone(),
                revision: 0,
                work_items: Vec::new(),
            });

            // 3. 验证 revision（乐观锁）
            if let Some(expected_revision) = command.expected_revision {
                if aggregate.revision != expected_revision {
                    return Err(format!(
                        "revision_conflict: expected {}, actual {}",
                        expected_revision, aggregate.revision
                    ));
                }
            }

            // 4. 查找 work item
            let work_item = aggregate
                .work_items
                .iter_mut()
                .find(|wi| wi.work_item_id == command.work_item_id);

            let work_item = match work_item {
                Some(wi) => wi,
                None => {
                    return Err(format!(
                        "work_item_not_found: work_item_id={}",
                        command.work_item_id
                    ));
                }
            };

            // 5. Policy 检查（真闸：control_core 状态转换表）
            if let Some(ref new_status) = command.new_status {
                let before_state = work_item.status.to_string();
                if let Err(reason) = crate::control_core::validate_work_item_state_transition(
                    &before_state,
                    &new_status.to_string(),
                ) {
                    // Policy-denied：写 scrubbed denial receipt，零 domain/event/outbox mutation
                    return create_denial_receipt_with_transaction(transaction, &command, &reason);
                }
            }

            // 6. 应用状态变更
            let old_status = work_item.status.clone();
            let old_state_json = work_item.state_json.clone();

            if let Some(new_status) = command.new_status.clone() {
                work_item.status = new_status;
            }
            if let Some(new_state_json) = command.new_state_json.clone() {
                work_item.state_json = new_state_json;
            }

            // 7. 递增 revision
            aggregate.revision += 1;

            // 8. 创建 Command Receipt
            let receipt = create_command_receipt(
                transaction,
                &command,
                &work_item.status,
                aggregate.revision,
            )?;

            // 9. 创建 Event
            let event = create_event(
                transaction,
                &command,
                &old_status,
                &work_item.status,
                aggregate.revision,
            )?;

            // 10. 创建 Audit Record
            let audit = create_audit_record(transaction, &command, &old_status, &work_item.status)?;

            // 11. Persist the sidecar revision only for the explicit M2 v1
            // command port.  Legacy callers deliberately carry no command
            // identity or expected revision; mutating their imported M1
            // sidecar-meta binding would turn a compatibility call into an
            // unannounced M2 migration.  The versioned port above always
            // supplies `Some(expected_revision)`, so it remains CAS-protected.
            if command.expected_revision.is_some() {
                save_aggregate(transaction, &aggregate)?;
            }

            // 12. 创建或更新 snapshot
            let snapshot = create_or_update_snapshot(transaction, &command, &event, &aggregate)?;

            // The JSON sidecar is a rebuildable internal projection.  M2
            // therefore records no external outbox item for this normal
            // product command; the caller validates parity after commit.

            Ok(UpdateWorkItemStateResult {
                receipt,
                event,
                audit,
                snapshot: Some(snapshot),
                outbox_item: None,
            })
        }
    }
}

/// 创建 Denial Receipt（使用 Transaction）
fn create_denial_receipt_with_transaction(
    transaction: &Transaction,
    command: &UpdateWorkItemStateCommand,
    reason: &str,
) -> Result<UpdateWorkItemStateResult, String> {
    // The receipt and audit rows are FK-bound to the narrow M2 command
    // registry. Keep that registry insert in the same denial UoW, so a
    // rejected explicit command remains append-only and cannot fail after
    // policy evaluation merely because the referenced identity is new.
    ensure_m2_command_and_correlation_registry(transaction, command)?;

    // 创建 denial receipt
    let receipt = CommandReceiptDto {
        receipt_id: generate_uuid(),
        command_id: command.command_id.clone(),
        idempotency_key: command.idempotency_key.clone(),
        request_hash: update_work_item_state_request_hash(command),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        current_object_ref: Some(format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        )),
        policy_decision_ref: m2_trace_policy_ref(
            command,
            format!("policy_gateway:denied:{}", reason),
        ),
        status: CommandReceiptStatus::Denied,
        correlation_id: Some(command.command_id.clone()),
        accepted_at: generate_timestamp(),
        result_ref: None,
        result_hash: None,
        committed_revision: None,
        error_code: Some("POLICY_DENIED".to_string()),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    transaction.execute(
        "INSERT INTO command_receipts (receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref, current_object_ref, policy_decision_ref, status, correlation_id, accepted_at, result_ref, result_hash, committed_revision, error_code, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        rusqlite::params![
            receipt.receipt_id,
            receipt.command_id,
            receipt.idempotency_key,
            receipt.request_hash,
            receipt.actor_id,
            receipt.scope_ref,
            receipt.current_object_ref,
            receipt.policy_decision_ref,
            receipt.status.to_string(),
            receipt.correlation_id,
            receipt.accepted_at,
            receipt.result_ref,
            receipt.result_hash,
            receipt.committed_revision,
            receipt.error_code,
            receipt.created_at,
        ],
    ).map_err(|e| format!("insert denial receipt failed: {}", e))?;

    // 创建 denial audit record（append-only，零 domain/event/outbox mutation）
    let audit = AuditRecordDto {
        audit_id: generate_uuid(),
        action: AuditAction::Denied,
        decision: format!("policy_denied: {}", reason),
        reason_code: Some("POLICY_DENIED".to_string()),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        subject_ref: Some(format!("work_item:{}", command.work_item_id)),
        command_id: Some(command.command_id.clone()),
        correlation_id: Some(command.command_id.clone()),
        occurred_at: generate_timestamp(),
        sensitivity: AuditSensitivity::Internal,
        scrub_result: Some("no_sensitive_material".to_string()),
        source_refs: Some(m2_trace_source_refs(command)),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    transaction.execute(
        "INSERT INTO audit_records (audit_id, action, decision, reason_code, actor_id, scope_ref, subject_ref, command_id, correlation_id, occurred_at, sensitivity, scrub_result, source_refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            audit.audit_id,
            audit.action.to_string(),
            audit.decision,
            audit.reason_code,
            audit.actor_id,
            audit.scope_ref,
            audit.subject_ref,
            audit.command_id,
            audit.correlation_id,
            audit.occurred_at,
            audit.sensitivity.to_string(),
            audit.scrub_result,
            audit.source_refs,
            audit.created_at,
        ],
    ).map_err(|e| format!("insert denial audit failed: {}", e))?;

    // 返回结果（零 domain/event/outbox mutation）
    Ok(UpdateWorkItemStateResult {
        receipt,
        event: WorkbenchEventEnvelopeDto {
            event_id: generate_uuid(),
            event_type: "WorkItemStateUpdateDenied".to_string(),
            occurred_at: generate_timestamp(),
            actor_id: command.actor_id.clone(),
            scope_ref: command.scope_ref.clone(),
            source_ref: format!(
                "workflow_state:{}:{}",
                command.project_id, command.workflow_id
            ),
            source_revision: None,
            command_id: Some(command.command_id.clone()),
            correlation_id: Some(command.command_id.clone()),
            causation_id: Some(command.command_id.clone()),
            trace_context: m2_workflow_state_sidecar_trace_context(command),
            schema_version: "1.0.0".to_string(),
            sensitivity: EventSensitivity::Internal,
            summary_ref: Some(format!(
                "work_item {} state update denied: {}",
                command.work_item_id, reason
            )),
            payload_ref: None,
            payload_hash: None,
            created_at: generate_timestamp(),
        },
        audit,
        snapshot: None,
        outbox_item: None,
    })
}

/// The M2 reference slice has one internal, post-commit effect: persist the
/// committed workflow state to the legacy JSON projection.  It is deliberately
/// not a generic outbox writer and cannot name an arbitrary capability.
pub(crate) const WORKFLOW_STATE_JSON_PROJECTION_CAPABILITY: &str =
    "workflow_state_json_projection.v1";
pub(crate) const WORKFLOW_STATE_JSON_PROJECTION_RESULT_COMMAND: &str =
    "workflow_state_projection_result.v1";

fn create_workflow_state_projection_outbox(
    connection: &Connection,
    command: &UpdateWorkItemStateCommand,
    receipt: &CommandReceiptDto,
    event: &WorkbenchEventEnvelopeDto,
) -> Result<OutboxItemDto, String> {
    let effect_material = format!(
        "{}:{}:{}:{}",
        WORKFLOW_STATE_JSON_PROJECTION_CAPABILITY,
        receipt.receipt_id,
        event.event_id,
        event.payload_hash.as_deref().unwrap_or_default(),
    );
    let effect_id = format!("workflow-state-projection:{}", sha256_hex(&effect_material));
    let outbox_item_id = format!("outbox:{}", sha256_hex(&effect_id));
    let item = OutboxItemDto {
        outbox_item_id,
        owning_command_id: command.command_id.clone(),
        owning_command_receipt_ref: receipt.receipt_id.clone(),
        effect_id,
        capability_id: WORKFLOW_STATE_JSON_PROJECTION_CAPABILITY.to_string(),
        scope_ref: command.scope_ref.clone(),
        subject_ref: Some(format!("work_item:{}", command.work_item_id)),
        payload_ref: event.payload_ref.clone(),
        payload_hash: event.payload_hash.clone(),
        result_command_type: WORKFLOW_STATE_JSON_PROJECTION_RESULT_COMMAND.to_string(),
        idempotency_key: format!("workflow-state-projection-result:{}", receipt.receipt_id),
        correlation_id: Some(command.command_id.clone()),
        status: OutboxItemStatus::Available,
        created_at: generate_timestamp(),
        expires_at: None,
        lease_token: None,
        claimer_id: None,
        acquired_at: None,
        attempt_count: Some(0),
        next_retry_not_before: None,
    };

    let inserted = connection
        .execute(
            "INSERT INTO outbox_items (
                outbox_item_id, owning_command_id, owning_command_receipt_ref,
                effect_id, capability_id, scope_ref, subject_ref, payload_ref,
                payload_hash, result_command_type, idempotency_key, correlation_id,
                status, created_at, expires_at, lease_token, claimer_id, acquired_at,
                attempt_count, next_retry_not_before
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                ?14, ?15, ?16, ?17, ?18, ?19, ?20
             )",
            rusqlite::params![
                item.outbox_item_id,
                item.owning_command_id,
                item.owning_command_receipt_ref,
                item.effect_id,
                item.capability_id,
                item.scope_ref,
                item.subject_ref,
                item.payload_ref,
                item.payload_hash,
                item.result_command_type,
                item.idempotency_key,
                item.correlation_id,
                item.status.to_string(),
                item.created_at,
                item.expires_at,
                item.lease_token,
                item.claimer_id,
                item.acquired_at,
                item.attempt_count,
                item.next_retry_not_before,
            ],
        )
        .map_err(|error| format!("insert workflow-state projection outbox failed: {error}"))?;
    if inserted != 1 {
        return Err("workflow_state_projection_outbox_insert_not_exactly_one".to_string());
    }
    Ok(item)
}

/// 检查幂等键（使用 Transaction）
fn check_idempotency_with_transaction(
    transaction: &Transaction,
    command: &UpdateWorkItemStateCommand,
    request_hash: &str,
) -> Result<IdempotencyResult, String> {
    let mut stmt = transaction
        .prepare(
            "SELECT receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref,
                current_object_ref, policy_decision_ref, status, correlation_id, accepted_at,
                result_ref, result_hash, committed_revision, error_code, created_at
         FROM command_receipts
         WHERE command_id = ?1 AND idempotency_key = ?2",
        )
        .map_err(|e| format!("prepare idempotency check failed: {}", e))?;

    let result = stmt
        .query_row(
            rusqlite::params![command.command_id, command.idempotency_key],
            |row| {
                Ok(CommandReceiptDto {
                    receipt_id: row.get(0)?,
                    command_id: row.get(1)?,
                    idempotency_key: row.get(2)?,
                    request_hash: row.get(3)?,
                    actor_id: row.get(4)?,
                    scope_ref: row.get(5)?,
                    current_object_ref: row.get(6)?,
                    policy_decision_ref: row.get(7)?,
                    status: match row.get::<_, String>(8)?.as_str() {
                        "DENIED" => CommandReceiptStatus::Denied,
                        "NEEDS_CONFIRMATION" => CommandReceiptStatus::NeedsConfirmation,
                        "COMMITTED" => CommandReceiptStatus::Committed,
                        "EXTERNAL_PENDING" => CommandReceiptStatus::ExternalPending,
                        "EXTERNAL_RESULT" => CommandReceiptStatus::ExternalResult,
                        "PROJECTION_DEGRADED" => CommandReceiptStatus::ProjectionDegraded,
                        "FAILED" => CommandReceiptStatus::Failed,
                        _ => CommandReceiptStatus::Failed,
                    },
                    correlation_id: row.get(9)?,
                    accepted_at: row.get(10)?,
                    result_ref: row.get(11)?,
                    result_hash: row.get(12)?,
                    committed_revision: row.get(13)?,
                    error_code: row.get(14)?,
                    created_at: row.get(15)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("query idempotency failed: {}", e))?;

    match result {
        Some(existing_receipt) => {
            // 检查 request_hash 是否匹配
            if existing_receipt.request_hash == request_hash {
                // 同 command_id + idempotency_key + 相同 request_hash → 返回既有 receipt
                Ok(IdempotencyResult::AlreadyProcessed(existing_receipt))
            } else {
                // 同键不同 hash → 报 conflict 错误
                Ok(IdempotencyResult::Conflict {
                    existing_hash: existing_receipt.request_hash,
                })
            }
        }
        None => {
            // 新请求
            Ok(IdempotencyResult::New)
        }
    }
}

/// 获取聚合（使用 Transaction）：从 work_items 表读取目标 work item（按主键查询，与 repository 一致）
fn get_aggregate_with_transaction(
    transaction: &Transaction,
    project_id: &str,
    workflow_id: &str,
    work_item_id: &str,
) -> Result<Option<WorkflowStateAggregate>, String> {
    let row: Option<(String, Option<String>, String)> = transaction
        .query_row(
            "SELECT work_item_id, node_id, record_json FROM work_items WHERE work_item_id = ?1",
            [work_item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|e| format!("get_aggregate: query failed: {}", e))?;

    let work_items: Vec<WorkItem> = match row {
        Some((wi_id, node_id, record_json)) => {
            let value: Value =
                serde_json::from_str(&record_json).unwrap_or_else(|_| Value::Object(Map::new()));
            let status_str = value
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("draft");
            vec![WorkItem {
                work_item_id: wi_id,
                node_id: node_id.unwrap_or_default(),
                status: WorkItemStatus::from_str(status_str),
                state_json: record_json,
            }]
        }
        None => Vec::new(),
    };

    // 获取 revision（从 workflow_state_meta 或默认 0）
    let revision: i64 = transaction
        .query_row(
            "SELECT revision FROM workflow_state_meta WHERE workspace_id = ?1 LIMIT 1",
            [workflow_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(Some(WorkflowStateAggregate {
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        revision,
        work_items,
    }))
}

/// 保存聚合（使用 Transaction）
fn save_aggregate_with_transaction(
    transaction: &Transaction,
    aggregate: &WorkflowStateAggregate,
) -> Result<(), String> {
    save_aggregate(transaction, aggregate)
}

/// 创建 Command Receipt（使用 Transaction）
fn create_command_receipt_with_transaction(
    transaction: &Transaction,
    command: &UpdateWorkItemStateCommand,
    new_status: &WorkItemStatus,
    revision: i64,
) -> Result<CommandReceiptDto, String> {
    let receipt = CommandReceiptDto {
        receipt_id: generate_uuid(),
        command_id: command.command_id.clone(),
        idempotency_key: command.idempotency_key.clone(),
        request_hash: update_work_item_state_request_hash(command),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        current_object_ref: Some(format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        )),
        policy_decision_ref: m2_trace_policy_ref(command, "policy_gateway:allowed".to_string()),
        status: CommandReceiptStatus::Committed,
        correlation_id: Some(command.command_id.clone()),
        accepted_at: generate_timestamp(),
        result_ref: Some(format!("work_item:{}:{}", command.work_item_id, revision)),
        result_hash: Some(sha256_hex(&new_status.to_string())),
        committed_revision: Some(revision),
        error_code: None,
        created_at: generate_timestamp(),
    };

    // 插入数据库
    transaction.execute(
        "INSERT INTO command_receipts (receipt_id, command_id, idempotency_key, request_hash, actor_id, scope_ref, current_object_ref, policy_decision_ref, status, correlation_id, accepted_at, result_ref, result_hash, committed_revision, error_code, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        rusqlite::params![
            receipt.receipt_id,
            receipt.command_id,
            receipt.idempotency_key,
            receipt.request_hash,
            receipt.actor_id,
            receipt.scope_ref,
            receipt.current_object_ref,
            receipt.policy_decision_ref,
            receipt.status.to_string(),
            receipt.correlation_id,
            receipt.accepted_at,
            receipt.result_ref,
            receipt.result_hash,
            receipt.committed_revision,
            receipt.error_code,
            receipt.created_at,
        ],
    ).map_err(|e| format!("insert receipt failed: {}", e))?;

    Ok(receipt)
}

/// 创建 Event（使用 Transaction）
fn create_event_with_transaction(
    transaction: &Transaction,
    command: &UpdateWorkItemStateCommand,
    old_status: &WorkItemStatus,
    new_status: &WorkItemStatus,
    revision: i64,
) -> Result<WorkbenchEventEnvelopeDto, String> {
    let event = WorkbenchEventEnvelopeDto {
        event_id: generate_uuid(),
        event_type: "WorkItemStateUpdated".to_string(),
        occurred_at: generate_timestamp(),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        source_ref: format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        ),
        source_revision: Some(revision.to_string()),
        command_id: Some(command.command_id.clone()),
        correlation_id: Some(command.command_id.clone()),
        causation_id: Some(command.command_id.clone()),
        trace_context: m2_workflow_state_sidecar_trace_context(command),
        schema_version: "1.0.0".to_string(),
        sensitivity: EventSensitivity::Internal,
        summary_ref: Some(format!(
            "work_item {} status {} -> {}",
            command.work_item_id, old_status, new_status
        )),
        payload_ref: Some(format!("work_item:{}:{}", command.work_item_id, revision)),
        payload_hash: Some(sha256_hex(&new_status.to_string())),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    transaction.execute(
        "INSERT INTO events (event_id, event_type, occurred_at, actor_id, scope_ref, source_ref, source_revision, command_id, correlation_id, causation_id, trace_context, schema_version, sensitivity, summary_ref, payload_ref, payload_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        rusqlite::params![
            event.event_id,
            event.event_type,
            event.occurred_at,
            event.actor_id,
            event.scope_ref,
            event.source_ref,
            event.source_revision,
            event.command_id,
            event.correlation_id,
            event.causation_id,
            event.trace_context,
            event.schema_version,
            event.sensitivity.to_string(),
            event.summary_ref,
            event.payload_ref,
            event.payload_hash,
            event.created_at,
        ],
    ).map_err(|e| format!("insert event failed: {}", e))?;

    Ok(event)
}

/// 创建 Audit Record（使用 Transaction）
fn create_audit_record_with_transaction(
    transaction: &Transaction,
    command: &UpdateWorkItemStateCommand,
    old_status: &WorkItemStatus,
    new_status: &WorkItemStatus,
) -> Result<AuditRecordDto, String> {
    let audit = AuditRecordDto {
        audit_id: generate_uuid(),
        action: AuditAction::Committed,
        decision: format!(
            "work_item {} updated: status {} -> {}",
            command.work_item_id, old_status, new_status
        ),
        reason_code: Some("policy_allowed".to_string()),
        actor_id: command.actor_id.clone(),
        scope_ref: command.scope_ref.clone(),
        subject_ref: Some(format!("work_item:{}", command.work_item_id)),
        command_id: Some(command.command_id.clone()),
        correlation_id: Some(command.command_id.clone()),
        occurred_at: generate_timestamp(),
        sensitivity: AuditSensitivity::Internal,
        scrub_result: Some("no_sensitive_material".to_string()),
        source_refs: Some(m2_trace_source_refs(command)),
        created_at: generate_timestamp(),
    };

    // 插入数据库
    transaction.execute(
        "INSERT INTO audit_records (audit_id, action, decision, reason_code, actor_id, scope_ref, subject_ref, command_id, correlation_id, occurred_at, sensitivity, scrub_result, source_refs, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
            audit.audit_id,
            audit.action.to_string(),
            audit.decision,
            audit.reason_code,
            audit.actor_id,
            audit.scope_ref,
            audit.subject_ref,
            audit.command_id,
            audit.correlation_id,
            audit.occurred_at,
            audit.sensitivity.to_string(),
            audit.scrub_result,
            audit.source_refs,
            audit.created_at,
        ],
    ).map_err(|e| format!("insert audit failed: {}", e))?;

    Ok(audit)
}

/// 创建或更新 Snapshot（使用 Transaction）
fn create_or_update_snapshot_with_transaction(
    transaction: &Transaction,
    command: &UpdateWorkItemStateCommand,
    event: &WorkbenchEventEnvelopeDto,
    aggregate: &WorkflowStateAggregate,
) -> Result<CurrentSnapshotDto, String> {
    let snapshot = CurrentSnapshotDto {
        object_ref: format!(
            "workflow_state:{}:{}",
            command.project_id, command.workflow_id
        ),
        object_revision: aggregate.revision,
        source_watermark: event.event_id.clone(),
        snapshot_hash: canonical_workflow_state_sidecar_snapshot_hash_for_aggregate(
            command, aggregate,
        ),
        projector_id: "workflow_projector".to_string(),
        built_at: generate_timestamp(),
    };

    // 插入或更新数据库
    transaction.execute(
        "INSERT OR REPLACE INTO current_snapshots (object_ref, object_revision, source_watermark, snapshot_hash, projector_id, built_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            snapshot.object_ref,
            snapshot.object_revision,
            snapshot.source_watermark,
            snapshot.snapshot_hash,
            snapshot.projector_id,
            snapshot.built_at,
        ],
    ).map_err(|e| format!("upsert snapshot failed: {}", e))?;

    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;

    // This file used to sit beside the compiled module without being linked
    // into the library test tree.  Keep it as a second, real fixture consumer
    // of the same exact scratch schema rather than treating it as evidence by
    // filename alone.
    include!("m2_update_work_item_state_tests.rs");

    fn temp_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "syn-m2-wiring-test-{}-{}",
            name,
            std::process::id()
        ));
        path
    }

    fn create_test_db(path: &Path) -> Connection {
        let connection = Connection::open(path).expect("open db");
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .expect("enable foreign keys");

        // Use the same versioned scratch schema as the concrete port.  A
        // hand-written subset used to omit the registry FK targets and made
        // these tests accidentally exercise a schema that production refuses.
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version TEXT PRIMARY KEY,
                    applied_at TEXT NOT NULL,
                    description TEXT NOT NULL
                )",
            )
            .expect("create schema_migrations table");
        crate::workbench_sqlite_schema_m2::apply_m2_schema(&connection)
            .expect("apply exact M2 scratch schema");

        // work_items 表（check_policy 和 get_aggregate 需要）
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS work_items (
                work_item_id TEXT PRIMARY KEY,
                workflow_id TEXT,
                node_id TEXT,
                source_id TEXT,
                record_hash TEXT NOT NULL,
                record_json TEXT NOT NULL
            )",
            )
            .expect("create work_items table");

        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS workflow_state_meta (
                workspace_id TEXT NOT NULL,
                source_root_hash TEXT NOT NULL,
                schema_version TEXT NOT NULL,
                workflow_version INTEGER NOT NULL,
                revision INTEGER,
                source_id TEXT,
                meta_json TEXT NOT NULL,
                PRIMARY KEY(workspace_id, source_root_hash)
            )",
            )
            .expect("create workflow_state_meta table");

        connection
    }

    /// 插入测试用 work_item 到 work_items 表（check_policy 通过此表读取当前状态）
    fn insert_test_work_item(
        connection: &Connection,
        work_item_id: &str,
        workflow_id: &str,
        state: &str,
    ) {
        let source_id = format!("m2-test-source:{workflow_id}");
        let workspace_id = format!("m2-test-workspace:{workflow_id}");
        let source_root_hash = format!("m2-test-root:{workflow_id}");
        let record_json = serde_json::json!({
            "work_item_id": work_item_id,
            "workflow_id": workflow_id,
            "state": state,
        })
        .to_string();
        connection.execute(
            "INSERT OR REPLACE INTO work_items (work_item_id, workflow_id, node_id, source_id, record_hash, record_json)
             VALUES (?1, ?2, 'node-001', ?3, 'hash', ?4)",
            rusqlite::params![work_item_id, workflow_id, source_id.as_str(), record_json],
        ).expect("insert test work_item");
        connection
            .execute(
                "INSERT OR REPLACE INTO workflow_state_meta
                 (workspace_id, source_root_hash, schema_version, workflow_version, revision, source_id, meta_json)
                 VALUES (?1, ?2, 'workflow_state_v0', 1, 0, ?3, '{\"revision\":0}')",
                rusqlite::params![workspace_id, source_root_hash, source_id.as_str()],
            )
            .expect("insert test workflow-state meta");
    }

    #[test]
    fn canonical_sidecar_snapshot_hash_tracks_content_not_json_key_order() {
        let work_item_a = json!({
            "work_item_id": "item-1",
            "state": "running",
            "state_json": {"priority": 1, "labels": ["m2", "slice"]}
        });
        let work_item_b = json!({
            "state_json": {"labels": ["m2", "slice"], "priority": 1},
            "state": "running",
            "work_item_id": "item-1"
        });
        let node_a = json!({"node_id": "node-1", "state": "running"});
        let node_b = json!({"state": "running", "node_id": "node-1"});
        let first = canonical_workflow_state_sidecar_snapshot_hash(
            "project-1",
            "workflow-1",
            7,
            &work_item_a,
            &node_a,
        );
        let rebuilt = canonical_workflow_state_sidecar_snapshot_hash(
            "project-1",
            "workflow-1",
            7,
            &work_item_b,
            &node_b,
        );
        let changed = canonical_workflow_state_sidecar_snapshot_hash(
            "project-1",
            "workflow-1",
            7,
            &json!({"work_item_id": "item-1", "state": "completed"}),
            &node_a,
        );
        assert_eq!(
            first, rebuilt,
            "canonical rebuild must retain the content hash"
        );
        assert_ne!(
            first, changed,
            "semantic content changes must change the hash"
        );
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn test_update_work_item_state_allowed() {
        let dir = temp_dir("allowed");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        // 插入 work_item，当前状态 draft → 允许转换到 ready_to_dispatch
        insert_test_work_item(&connection, "work-item-001", "workflow-001", "draft");

        let command = UpdateWorkItemStateCommand {
            command_id: "cmd-001".to_string(),
            idempotency_key: "idem-001".to_string(),
            actor_id: "user-001".to_string(),
            scope_ref: "scope-001".to_string(),
            project_id: "project-001".to_string(),
            workflow_id: "workflow-001".to_string(),
            work_item_id: "work-item-001".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::ReadyToDispatch),
            new_state_json: None,
        };

        let result = update_work_item_state_m2(&connection, command);
        assert!(
            result.is_ok(),
            "update_work_item_state_m2 failed: {:?}",
            result.err()
        );

        let result = result.unwrap();
        assert_eq!(result.receipt.status, CommandReceiptStatus::Committed);
        assert_eq!(result.event.event_type, "WorkItemStateUpdated");
        assert_eq!(result.audit.action, AuditAction::Committed);
        assert!(result.snapshot.is_some());
        assert!(
            result.outbox_item.is_none(),
            "the rebuildable JSON projection must not masquerade as an external effect"
        );

        // 验证数据库中有记录
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM audit_records", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM outbox_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM current_snapshots", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_update_work_item_state_denied() {
        let dir = temp_dir("denied");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        // 插入 work_item，当前状态 draft → 非法转换到 failed（draft 只能到 ready_to_dispatch）
        insert_test_work_item(&connection, "work-item-002", "workflow-002", "draft");

        let command = UpdateWorkItemStateCommand {
            command_id: "cmd-002".to_string(),
            idempotency_key: "idem-002".to_string(),
            actor_id: "user-002".to_string(),
            scope_ref: "scope-002".to_string(),
            project_id: "project-002".to_string(),
            workflow_id: "workflow-002".to_string(),
            work_item_id: "work-item-002".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::Failed),
            new_state_json: None,
        };

        let result = update_work_item_state_m2(&connection, command);
        assert!(
            result.is_ok(),
            "denied path should return Ok with denial receipt: {:?}",
            result.err()
        );

        let result = result.unwrap();
        // 验证 receipt 是 denial 状态
        assert_eq!(result.receipt.status, CommandReceiptStatus::Denied);
        // 验证 denial audit record 落盘
        let audit_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM audit_records", [], |row| row.get(0))
            .unwrap();
        assert_eq!(audit_count, 1);
        // 验证零业务变化：events 表无新行
        let event_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(event_count, 0);
        let outbox_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM outbox_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(outbox_count, 0, "denial must not declare an effect");
    }

    #[test]
    fn test_idempotency_same_key_same_hash_returns_same_receipt() {
        let dir = temp_dir("idem-same");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        insert_test_work_item(&connection, "work-item-003", "workflow-003", "draft");

        let command = UpdateWorkItemStateCommand {
            command_id: "cmd-003".to_string(),
            idempotency_key: "idem-003".to_string(),
            actor_id: "user-003".to_string(),
            scope_ref: "scope-003".to_string(),
            project_id: "project-003".to_string(),
            workflow_id: "workflow-003".to_string(),
            work_item_id: "work-item-003".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::ReadyToDispatch),
            new_state_json: None,
        };

        // 第一次执行
        let result1 = update_work_item_state_m2(&connection, command.clone()).unwrap();
        assert_eq!(result1.receipt.status, CommandReceiptStatus::Committed);

        // The receipt must carry the exact complete-payload hash used by both
        // entrypoint preflight and the UoW.
        let stored_hash: String = connection
            .query_row(
                "SELECT request_hash FROM command_receipts WHERE command_id = ?1",
                ["cmd-003"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            stored_hash,
            update_work_item_state_request_hash(&command),
            "stored receipt must use the complete canonical command payload"
        );

        // 验证数据库中有 1 条 receipt
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);

        let outbox_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM outbox_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            outbox_count, 0,
            "idempotent replay must not invent an external projection effect"
        );

        // 第二次执行（相同 command_id + idempotency_key + 相同 request_hash）
        let result2 = update_work_item_state_m2(&connection, command).unwrap();
        // 返回既有 receipt（receipt_id 相同）
        assert_eq!(result2.receipt.receipt_id, result1.receipt.receipt_id);

        // 验证数据库中仍然只有 1 条 receipt（幂等，不新增行）
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM command_receipts", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_idempotency_same_key_different_hash_returns_conflict() {
        let dir = temp_dir("idem-conflict");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        insert_test_work_item(&connection, "work-item-004", "workflow-004", "draft");

        // 第一次执行
        let command1 = UpdateWorkItemStateCommand {
            command_id: "cmd-004".to_string(),
            idempotency_key: "idem-004".to_string(),
            actor_id: "user-004".to_string(),
            scope_ref: "scope-004".to_string(),
            project_id: "project-004".to_string(),
            workflow_id: "workflow-004".to_string(),
            work_item_id: "work-item-004".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::ReadyToDispatch),
            new_state_json: None,
        };
        let result1 = update_work_item_state_m2(&connection, command1).unwrap();
        assert_eq!(result1.receipt.status, CommandReceiptStatus::Committed);

        // 第二次执行（相同 command_id + idempotency_key + 不同 request_hash → 不同 new_status）
        let command2 = UpdateWorkItemStateCommand {
            command_id: "cmd-004".to_string(),
            idempotency_key: "idem-004".to_string(),
            actor_id: "user-004".to_string(),
            scope_ref: "scope-004".to_string(),
            project_id: "project-004".to_string(),
            workflow_id: "workflow-004".to_string(),
            work_item_id: "work-item-004".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::Running), // 不同的 new_status → 不同的 request_hash
            new_state_json: None,
        };
        let result2 = update_work_item_state_m2(&connection, command2);
        assert!(
            result2.is_err(),
            "different hash should return conflict error"
        );
        assert!(result2.unwrap_err().contains("idempotent_conflict"));
    }

    #[test]
    fn canonical_request_hash_separates_actor_scope_revision_and_payload() {
        let base = UpdateWorkItemStateCommand {
            command_id: "cmd-canonical-payload".to_string(),
            idempotency_key: "idem-canonical-payload".to_string(),
            actor_id: "actor-a".to_string(),
            scope_ref: "scope-a".to_string(),
            project_id: "project-a".to_string(),
            workflow_id: "workflow-a".to_string(),
            work_item_id: "work-item-a".to_string(),
            expected_revision: Some(7),
            new_status: Some(WorkItemStatus::Running),
            new_state_json: Some("{\"repository_port_version\":\"workflow-state-sidecar.repository.m2.v1\",\"after_state\":\"running\"}".to_string()),
        };
        let base_hash = update_work_item_state_request_hash(&base);

        let mut changed_actor = base.clone();
        changed_actor.actor_id = "actor-b".to_string();
        assert_ne!(
            base_hash,
            update_work_item_state_request_hash(&changed_actor)
        );

        let mut changed_scope = base.clone();
        changed_scope.scope_ref = "scope-b".to_string();
        assert_ne!(
            base_hash,
            update_work_item_state_request_hash(&changed_scope)
        );

        let mut changed_revision = base.clone();
        changed_revision.expected_revision = Some(8);
        assert_ne!(
            base_hash,
            update_work_item_state_request_hash(&changed_revision)
        );

        let mut changed_payload = base.clone();
        changed_payload.new_state_json = Some("{\"after_state\":\"running\",\"repository_port_version\":\"workflow-state-sidecar.repository.m2.v2\"}".to_string());
        assert_ne!(
            base_hash,
            update_work_item_state_request_hash(&changed_payload)
        );

        let mut reordered_equivalent_payload = base;
        reordered_equivalent_payload.new_state_json = Some("{\"after_state\":\"running\",\"repository_port_version\":\"workflow-state-sidecar.repository.m2.v1\"}".to_string());
        assert_eq!(
            base_hash,
            update_work_item_state_request_hash(&reordered_equivalent_payload),
            "JSON formatting/order must not create a different logical command"
        );
    }
}
