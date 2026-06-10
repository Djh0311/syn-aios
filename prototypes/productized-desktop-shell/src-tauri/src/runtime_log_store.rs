use crate::{
    ControlledSessionContinuation, RuntimeLogBoundary, RuntimeLogEntry, RuntimeLogSourceRef,
    RuntimeLogStoreScope, RuntimeLogStoreV1, RuntimeLogSummary, RuntimeSessionAttention,
    SessionContinuationAttempt, SessionContinuationStoreV1,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "runtime_log_store.v1";
const STORAGE_KIND: &str = "sidecar_json_v0";
const SIDECAR_NAME: &str = "runtime-logs.v1.json";
const LOCK_NAME: &str = ".runtime-logs.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导 runtime log sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store_or_derive(
    workflow_state_path: &Path,
    continuation_store: &SessionContinuationStoreV1,
    attention: &[RuntimeSessionAttention],
    timestamp: &str,
) -> RuntimeLogStoreV1 {
    match load_store(workflow_state_path) {
        Ok(store) => store,
        Err(warning) => {
            let mut store = derive_store_from_runtime_state(
                &workflow_state_path.display().to_string(),
                continuation_store,
                attention,
                timestamp,
            );
            if warning != "runtime_log_sidecar_missing" {
                store.warnings.push(warning);
            }
            store
        }
    }
}

pub(crate) fn load_store(workflow_state_path: &Path) -> Result<RuntimeLogStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Err("runtime_log_sidecar_missing".to_string());
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "读取 runtime log sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let mut store: RuntimeLogStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "runtime log sidecar JSON 损坏，已拒绝展示 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    store.entries = store.entries.into_iter().map(redact_entry).collect();
    store.summaries = summarize_entries(&store.entries);
    Ok(store)
}

pub(crate) fn ensure_appendable(workflow_state_path: &Path) -> Result<(), String> {
    match load_store(workflow_state_path) {
        Ok(_) => Ok(()),
        Err(error) if error == "runtime_log_sidecar_missing" => Ok(()),
        Err(error) => Err(format!(
            "runtime_log_sidecar_unreadable_refuse_h2_attempt: {error}"
        )),
    }
}

pub(crate) fn append_session_continuation_attempt(
    workflow_state_path: &Path,
    continuation_store: &SessionContinuationStoreV1,
    continuation: &ControlledSessionContinuation,
    attempt: &SessionContinuationAttempt,
    timestamp: &str,
    write_id: &str,
) -> Result<RuntimeLogStoreV1, String> {
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
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = match load_store(workflow_state_path) {
        Ok(store) => store,
        Err(error) if error == "runtime_log_sidecar_missing" => {
            empty_store(workflow_state_path, &sidecar, continuation_store, timestamp)
        }
        Err(error) => {
            drop(lock);
            return Err(format!(
                "runtime_log_sidecar_unreadable_refuse_overwrite: {error}"
            ));
        }
    };

    let entries = vec![
        workflow_run_entry(continuation),
        dispatch_attempt_entry(attempt, Some(continuation)),
        readback_entry(attempt, Some(continuation)),
    ];
    let entry_ids = entries
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect::<BTreeSet<_>>();
    store
        .entries
        .retain(|entry| !entry_ids.contains(&entry.entry_id));
    store.entries.extend(entries.into_iter().map(redact_entry));
    store.entries.sort_by(|left, right| {
        left.started_at
            .cmp(&right.started_at)
            .then_with(|| left.entry_id.cmp(&right.entry_id))
    });
    store.scope.project_roots = merge_project_roots(
        store.scope.project_roots,
        continuation_store.scope.project_roots.clone(),
        continuation.project_root.clone(),
    );
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.generated_by = "runtime_log_store_explicit_append_v1".to_string();
    store.updated_at = timestamp.to_string();
    store.summaries = summarize_entries(&store.entries);
    store.warnings = merge_warnings(
        store.warnings,
        vec![
            "runtime_log_sidecar_explicitly_written".to_string(),
            "runtime_log_does_not_replace_audit_event".to_string(),
            "audit_event_does_not_replace_runtime_log".to_string(),
        ],
    );
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);
    Ok(store)
}

pub(crate) fn derive_store_from_runtime_state(
    workflow_state_path: &str,
    continuation_store: &SessionContinuationStoreV1,
    attention: &[RuntimeSessionAttention],
    timestamp: &str,
) -> RuntimeLogStoreV1 {
    let sidecar = Path::new(workflow_state_path)
        .parent()
        .map(|parent| parent.join(SIDECAR_NAME).display().to_string());
    let mut entries = vec![RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:app-session:{timestamp}"),
        category: "app_session".to_string(),
        status: "observed".to_string(),
        severity: "info".to_string(),
        started_at: Some(timestamp.to_string()),
        finished_at: None,
        duration_ms: None,
        project_id: None,
        workflow_id: None,
        node_id: None,
        session_id: None,
        adapter_id: None,
        summary: "Workbench app session runtime summary is available.".to_string(),
        detail:
            "Runtime log records operational status only; audit events remain separate records."
                .to_string(),
        source_refs: vec![],
        audit_refs: vec![],
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec![],
        user_visible: true,
        warnings: vec!["runtime_log_is_not_audit_event".to_string()],
    }];

    for continuation in &continuation_store.continuations {
        entries.push(workflow_run_entry(continuation));
        if continuation.status == "preview_confirmed"
            || continuation.status == "queued"
            || continuation.status == "waiting_permission"
            || continuation.user_confirmation_state != "confirmed"
        {
            entries.push(permission_wait_entry(continuation));
        }
    }

    for attempt in &continuation_store.attempts {
        let continuation = continuation_store
            .continuations
            .iter()
            .find(|item| item.continuation_id == attempt.continuation_id);
        entries.push(dispatch_attempt_entry(attempt, continuation));
        entries.push(readback_entry(attempt, continuation));
    }

    for item in attention {
        if item.kind.contains("permission") || item.requires_user_action {
            entries.push(permission_wait_attention_entry(item));
        }
        if item.kind.contains("readback") || item.readback_boundary.status.contains("readback") {
            entries.push(readback_attention_entry(item));
        }
    }

    entries.push(RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:diagnostic:{timestamp}"),
        category: "diagnostic_event".to_string(),
        status: "summary_available".to_string(),
        severity: if continuation_store.warnings.is_empty() {
            "info".to_string()
        } else {
            "warning".to_string()
        },
        started_at: Some(timestamp.to_string()),
        finished_at: Some(timestamp.to_string()),
        duration_ms: None,
        project_id: None,
        workflow_id: None,
        node_id: None,
        session_id: None,
        adapter_id: None,
        summary: format!(
            "Runtime log diagnostic summary: continuations {} / attempts {} / attention {}.",
            continuation_store.continuations.len(),
            continuation_store.attempts.len(),
            attention.len()
        ),
        detail: "Diagnostic runtime entries summarize store readability and derived runtime state; G2 owns full diagnostics."
            .to_string(),
        source_refs: vec![RuntimeLogSourceRef {
            source_kind: "runtime_log_derivation".to_string(),
            source_id: "workbench_snapshot".to_string(),
            label: "workbench snapshot".to_string(),
        }],
        audit_refs: vec![],
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec![],
        user_visible: true,
        warnings: vec!["g2_full_diagnostics_deferred".to_string()],
    });

    let entries: Vec<_> = entries.into_iter().map(redact_entry).collect();
    RuntimeLogStoreV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        store_version: 1,
        storage_kind: STORAGE_KIND.to_string(),
        scope: RuntimeLogStoreScope {
            scope_kind: "workflow_state_sidecar".to_string(),
            workflow_state_path: Some(workflow_state_path.to_string()),
            sidecar_path: sidecar,
            project_roots: continuation_store.scope.project_roots.clone(),
        },
        revision: 0,
        last_write_id: None,
        generated_by: "runtime_log_store_derive_v1".to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        boundary: boundary(),
        summaries: summarize_entries(&entries),
        entries,
        warnings: vec![
            "runtime_log_store_derived_from_workbench_state".to_string(),
            "runtime_log_does_not_replace_audit_event".to_string(),
            "audit_event_does_not_replace_runtime_log".to_string(),
        ],
    }
}

fn workflow_run_entry(continuation: &ControlledSessionContinuation) -> RuntimeLogEntry {
    RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:workflow-run:{}", continuation.continuation_id),
        category: "workflow_run".to_string(),
        status: continuation.status.clone(),
        severity: severity_for_status(&continuation.status),
        started_at: Some(continuation.created_at.clone()),
        finished_at: Some(continuation.updated_at.clone()),
        duration_ms: None,
        project_id: Some(continuation.project_id.clone()),
        workflow_id: Some(continuation.workflow_id.clone()),
        node_id: Some(continuation.node_id.clone()),
        session_id: Some(continuation.session_id.clone()),
        adapter_id: Some(continuation.adapter_id.clone()),
        summary: format!(
            "Workflow run {} is {} via {}.",
            continuation.workflow_id, continuation.status, continuation.runner_kind
        ),
        detail: "Runtime log keeps run status and refs; command and prompt bodies are omitted."
            .to_string(),
        source_refs: vec![RuntimeLogSourceRef {
            source_kind: "controlled_session_continuation".to_string(),
            source_id: continuation.continuation_id.clone(),
            label: "controlled continuation".to_string(),
        }],
        audit_refs: sanitize_refs(&continuation.audit_refs),
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec!["command_body".to_string(), "prompt_body".to_string()],
        user_visible: true,
        warnings: vec![],
    }
}

fn dispatch_attempt_entry(
    attempt: &SessionContinuationAttempt,
    continuation: Option<&ControlledSessionContinuation>,
) -> RuntimeLogEntry {
    RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:dispatch-attempt:{}", attempt.attempt_id),
        category: "dispatch_attempt".to_string(),
        status: attempt.status.clone(),
        severity: severity_for_status(&attempt.status),
        started_at: Some(attempt.started_at.clone()),
        finished_at: attempt.finished_at.clone(),
        duration_ms: attempt.timeout_ms,
        project_id: continuation.map(|item| item.project_id.clone()),
        workflow_id: continuation.map(|item| item.workflow_id.clone()),
        node_id: continuation.map(|item| item.node_id.clone()),
        session_id: continuation.map(|item| item.session_id.clone()),
        adapter_id: continuation.map(|item| item.adapter_id.clone()),
        summary: format!(
            "Dispatch attempt {} ended as {}; runner {}.",
            attempt.attempt_id, attempt.status, attempt.runner_kind
        ),
        detail: "Dispatch runtime entry omits command preview, prompt text and raw runner output."
            .to_string(),
        source_refs: vec![RuntimeLogSourceRef {
            source_kind: "session_continuation_attempt".to_string(),
            source_id: attempt.attempt_id.clone(),
            label: "dispatch attempt".to_string(),
        }],
        audit_refs: sanitize_refs(&attempt.audit_refs),
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec!["command_body".to_string(), "runner_output".to_string()],
        user_visible: true,
        warnings: boundary_warnings(attempt),
    }
}

fn readback_entry(
    attempt: &SessionContinuationAttempt,
    continuation: Option<&ControlledSessionContinuation>,
) -> RuntimeLogEntry {
    RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:readback:{}", attempt.attempt_id),
        category: "readback".to_string(),
        status: attempt.readback_summary.status.clone(),
        severity: if attempt.readback_summary.status == "readback_unavailable" {
            "warning".to_string()
        } else {
            severity_for_status(&attempt.readback_summary.status)
        },
        started_at: attempt.finished_at.clone().or_else(|| Some(attempt.started_at.clone())),
        finished_at: attempt.finished_at.clone(),
        duration_ms: None,
        project_id: continuation.map(|item| item.project_id.clone()),
        workflow_id: continuation.map(|item| item.workflow_id.clone()),
        node_id: continuation.map(|item| item.node_id.clone()),
        session_id: continuation.map(|item| item.session_id.clone()),
        adapter_id: continuation.map(|item| item.adapter_id.clone()),
        summary: format!(
            "Readback status is {}; result_count={}.",
            attempt.readback_summary.status,
            attempt
                .readback_summary
                .result_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unavailable".to_string())
        ),
        detail: "Readback runtime entry never treats unavailable as zero results and omits conversation bodies."
            .to_string(),
        source_refs: vec![RuntimeLogSourceRef {
            source_kind: "session_continuation_attempt".to_string(),
            source_id: attempt.attempt_id.clone(),
            label: "readback boundary".to_string(),
        }],
        audit_refs: sanitize_refs(&attempt.audit_refs),
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec!["conversation_body".to_string(), "provider_material".to_string()],
        user_visible: true,
        warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
    }
}

fn permission_wait_entry(continuation: &ControlledSessionContinuation) -> RuntimeLogEntry {
    RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:permission-wait:{}", continuation.continuation_id),
        category: "permission_wait".to_string(),
        status: continuation.user_confirmation_state.clone(),
        severity: "warning".to_string(),
        started_at: Some(continuation.created_at.clone()),
        finished_at: None,
        duration_ms: None,
        project_id: Some(continuation.project_id.clone()),
        workflow_id: Some(continuation.workflow_id.clone()),
        node_id: Some(continuation.node_id.clone()),
        session_id: Some(continuation.session_id.clone()),
        adapter_id: Some(continuation.adapter_id.clone()),
        summary: "Permission wait is visible; no automatic retry or execution is implied.".to_string(),
        detail: "Permission runtime entry records waiting state only; permission decisions remain audited separately."
            .to_string(),
        source_refs: vec![RuntimeLogSourceRef {
            source_kind: "controlled_session_continuation".to_string(),
            source_id: continuation.continuation_id.clone(),
            label: "permission wait".to_string(),
        }],
        audit_refs: sanitize_refs(&continuation.audit_refs),
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec![],
        user_visible: true,
        warnings: vec!["no_auto_retry".to_string()],
    }
}

fn permission_wait_attention_entry(item: &RuntimeSessionAttention) -> RuntimeLogEntry {
    RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:permission-wait:{}", item.attention_id),
        category: "permission_wait".to_string(),
        status: item.status.clone(),
        severity: severity_for_status(&item.severity),
        started_at: Some(item.created_at.clone()),
        finished_at: Some(item.updated_at.clone()),
        duration_ms: None,
        project_id: item.project_id.clone(),
        workflow_id: item.workflow_id.clone(),
        node_id: item.node_id.clone(),
        session_id: item.session_id.clone(),
        adapter_id: Some(item.adapter_id.clone()),
        summary: format!("Runtime attention requires user review: {}.", item.status),
        detail: "Attention text is summarized; raw runtime material is omitted.".to_string(),
        source_refs: source_refs_from_attention(item),
        audit_refs: vec![],
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec!["attention_detail".to_string()],
        user_visible: true,
        warnings: vec![],
    }
}

fn readback_attention_entry(item: &RuntimeSessionAttention) -> RuntimeLogEntry {
    RuntimeLogEntry {
        entry_version: 1,
        entry_id: format!("runtime-log:readback-attention:{}", item.attention_id),
        category: "readback".to_string(),
        status: item.readback_boundary.status.clone(),
        severity: severity_for_status(&item.severity),
        started_at: Some(item.created_at.clone()),
        finished_at: Some(item.updated_at.clone()),
        duration_ms: None,
        project_id: item.project_id.clone(),
        workflow_id: item.workflow_id.clone(),
        node_id: item.node_id.clone(),
        session_id: item.session_id.clone(),
        adapter_id: Some(item.adapter_id.clone()),
        summary: format!(
            "Readback boundary attention: {} / {}.",
            item.readback_boundary.status, item.readback_boundary.reason
        ),
        detail: "Readback attention omits raw readback material and keeps unavailable distinct from zero results."
            .to_string(),
        source_refs: source_refs_from_attention(item),
        audit_refs: vec![],
        redaction_status: "redacted_safe_summary".to_string(),
        sensitive_omissions: vec!["readback_material".to_string()],
        user_visible: true,
        warnings: item.readback_boundary.warnings.clone(),
    }
}

fn source_refs_from_attention(item: &RuntimeSessionAttention) -> Vec<RuntimeLogSourceRef> {
    item.source_refs
        .iter()
        .map(|source| RuntimeLogSourceRef {
            source_kind: source.source_kind.clone(),
            source_id: sanitize_ref(&source.source_id),
            label: sanitize_text(&source.label),
        })
        .collect()
}

fn summarize_entries(entries: &[RuntimeLogEntry]) -> Vec<RuntimeLogSummary> {
    let mut grouped: BTreeMap<(String, String, String), Vec<&RuntimeLogEntry>> = BTreeMap::new();
    for entry in entries.iter().filter(|entry| entry.user_visible) {
        grouped
            .entry((
                entry.category.clone(),
                entry.status.clone(),
                entry.severity.clone(),
            ))
            .or_default()
            .push(entry);
    }
    grouped
        .into_iter()
        .map(|((category, status, severity), items)| RuntimeLogSummary {
            category,
            status,
            severity,
            entry_count: items.len(),
            latest_entry_ids: items
                .iter()
                .rev()
                .take(4)
                .map(|entry| entry.entry_id.clone())
                .collect(),
            warnings: vec![],
        })
        .collect()
}

fn validate_store(store: &RuntimeLogStoreV1) -> Result<(), String> {
    if store.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "runtime log schema_version 不匹配：{}",
            store.schema_version
        ));
    }
    if store.store_version != 1 {
        return Err(format!(
            "runtime log store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.storage_kind != STORAGE_KIND {
        return Err(format!(
            "runtime log storage_kind 不匹配：{}",
            store.storage_kind
        ));
    }
    Ok(())
}

fn redact_entry(mut entry: RuntimeLogEntry) -> RuntimeLogEntry {
    entry.entry_id = sanitize_ref(&entry.entry_id);
    entry.status = sanitize_text(&entry.status);
    entry.summary = sanitize_text(&entry.summary);
    entry.detail = sanitize_text(&entry.detail);
    entry.source_refs = entry
        .source_refs
        .into_iter()
        .map(|source| RuntimeLogSourceRef {
            source_kind: sanitize_text(&source.source_kind),
            source_id: sanitize_ref(&source.source_id),
            label: sanitize_text(&source.label),
        })
        .collect();
    entry.audit_refs = sanitize_refs(&entry.audit_refs);
    entry.warnings = entry
        .warnings
        .into_iter()
        .map(|warning| sanitize_text(&warning))
        .collect();
    entry.redaction_status = "redacted_safe_summary".to_string();
    entry
}

fn sanitize_refs(refs: &[String]) -> Vec<String> {
    refs.iter().map(|value| sanitize_ref(value)).collect()
}

fn sanitize_ref(value: &str) -> String {
    if contains_sensitive(value) {
        format!("runtime-ref:redacted:{}", stable_suffix(value))
    } else {
        value.to_string()
    }
}

fn sanitize_text(value: &str) -> String {
    if contains_sensitive(value) {
        "[redacted runtime summary]".to_string()
    } else {
        value.to_string()
    }
}

fn contains_sensitive(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "token",
        "secret",
        "oauth",
        "auth",
        ".env",
        "keychain",
        "credential",
        "sk-",
        "bearer ",
        "完整 transcript",
        "full transcript",
        "raw provider credential",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn stable_suffix(value: &str) -> String {
    let mut hash: u64 = 1469598103934665603;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:x}", hash)[..8].to_string()
}

fn severity_for_status(status: &str) -> String {
    if status.contains("blocked") || status.contains("failed") {
        "warning".to_string()
    } else if status.contains("timeout") || status.contains("unavailable") {
        "warning".to_string()
    } else {
        "info".to_string()
    }
}

fn boundary_warnings(attempt: &SessionContinuationAttempt) -> Vec<String> {
    let mut warnings = vec!["runtime_log_redacted_summary_only".to_string()];
    if !attempt.real_codex_executed {
        warnings.push("no_real_codex_execution_recorded".to_string());
    }
    if !attempt.prompt_sent {
        warnings.push("prompt_not_sent".to_string());
    }
    warnings
}

fn empty_store(
    workflow_state_path: &Path,
    sidecar: &Path,
    continuation_store: &SessionContinuationStoreV1,
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
            project_roots: continuation_store.scope.project_roots.clone(),
        },
        revision: 0,
        last_write_id: None,
        generated_by: "runtime_log_store_explicit_append_v1".to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        boundary: boundary(),
        entries: vec![],
        summaries: vec![],
        warnings: vec![
            "runtime_log_sidecar_explicitly_written".to_string(),
            "runtime_log_does_not_replace_audit_event".to_string(),
            "audit_event_does_not_replace_runtime_log".to_string(),
        ],
    }
}

fn merge_project_roots(
    existing: Vec<String>,
    next: Vec<String>,
    project_root: String,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    existing
        .into_iter()
        .chain(next)
        .chain(std::iter::once(project_root))
        .filter(|root| !root.trim().is_empty())
        .filter(|root| seen.insert(root.clone()))
        .collect()
}

fn merge_warnings(existing: Vec<String>, next: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    existing
        .into_iter()
        .chain(next)
        .filter(|warning| seen.insert(warning.clone()))
        .collect()
}

fn write_store_atomic(
    sidecar: &Path,
    store: &RuntimeLogStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("runtime log sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!(
                "创建 runtime log 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?;
        let backup = backup_dir.join(format!(
            "runtime-logs.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup).map_err(|error| {
            format!(
                "备份 runtime log sidecar 失败 {}：{error}",
                backup.display()
            )
        })?;
        prune_backups(&backup_dir, "runtime-logs.v1.")?;
    }
    let temp_path = parent.join(format!(".runtime-logs.v1.{timestamp}.{write_id}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("runtime log sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建 runtime log 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入 runtime log 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步 runtime log 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换 runtime log sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn prune_backups(backup_dir: &Path, prefix: &str) -> Result<(), String> {
    let mut backups = fs::read_dir(backup_dir)
        .map_err(|error| {
            format!(
                "读取 runtime log 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
        .collect::<Vec<_>>();
    backups.sort_by_key(|entry| entry.file_name());
    let remove_count = backups.len().saturating_sub(20);
    for entry in backups.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => {
                file.write_all(write_id.as_bytes()).map_err(|error| {
                    format!("写入 runtime log lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(format!("runtime_log_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建 runtime log lock 失败 {}：{error}",
                path.display()
            )),
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn boundary() -> RuntimeLogBoundary {
    RuntimeLogBoundary {
        runtime_log_definition:
            "Runtime log records operational timing, status, category, source refs and safe summaries."
                .to_string(),
        audit_event_definition:
            "Audit event records accountable decisions, actor, permission, before and after state."
                .to_string(),
        separation_rule:
            "Runtime log 与 audit event 不能互相替代；日志只引用 audit_refs，不内嵌 audit event 本体。"
                .to_string(),
        redaction_rule:
            "Only redacted summaries are displayable; sensitive material and conversation bodies are omitted."
                .to_string(),
        forbidden_payloads: vec![
            "credential_material".to_string(),
            "conversation_body".to_string(),
            "raw_provider_material".to_string(),
            "environment_material".to_string(),
            "system_authorization_material".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ReadbackBoundaryStatus, RuntimeAttentionSourceRef, SessionContinuationAttempt,
        SessionContinuationReadbackSummary, SessionContinuationStoreScope,
    };

    #[test]
    fn runtime_log_store_redacts_runtime_records_and_keeps_audit_as_refs() {
        let mut store = empty_continuation_store();
        store.continuations.push(ControlledSessionContinuation {
            record_version: 1,
            continuation_id: "continuation-secret".to_string(),
            preview_id: "preview-secret".to_string(),
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            project_id: "project-1".to_string(),
            project_root: "/tmp/project".to_string(),
            workflow_id: "workflow-1".to_string(),
            node_id: "node-1".to_string(),
            session_id: "session-1".to_string(),
            work_item_id: Some("work-item-1".to_string()),
            target_cwd: "/tmp/project".to_string(),
            allowed_write_roots: vec!["/tmp/project".to_string()],
            sandbox: "workspace-write".to_string(),
            prompt_source_kind: "workflow_followup".to_string(),
            prompt_summary: "use token sk-test-secret and full transcript".to_string(),
            command_preview: "codex exec resume sk-test-secret".to_string(),
            readback_strategy: "required".to_string(),
            status: "succeeded_stub".to_string(),
            execution_level: "level_a_stub_only".to_string(),
            runner_kind: "stub".to_string(),
            user_confirmation_state: "confirmed".to_string(),
            guard_status: "needs_user_confirmation".to_string(),
            requested_by: "user".to_string(),
            confirmed_by: "user".to_string(),
            confirmation_reason: "contains secret".to_string(),
            created_at: "2026-06-07T00:00:00Z".to_string(),
            updated_at: "2026-06-07T00:00:01Z".to_string(),
            audit_refs: vec!["audit:session-continuation-confirmed:secret".to_string()],
            warnings: vec![],
        });
        store.attempts.push(SessionContinuationAttempt {
            attempt_version: 1,
            attempt_id: "attempt-secret".to_string(),
            continuation_id: "continuation-secret".to_string(),
            runner_kind: "stub".to_string(),
            execution_level: "level_a_stub_only".to_string(),
            status: "succeeded_stub".to_string(),
            started_at: "2026-06-07T00:00:01Z".to_string(),
            finished_at: Some("2026-06-07T00:00:02Z".to_string()),
            timeout_ms: Some(1000),
            command_preview: "codex exec resume sk-test-secret".to_string(),
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_workbench_state: true,
            readback_summary: SessionContinuationReadbackSummary {
                status: "readback_unavailable".to_string(),
                source_kind: "stub_no_transcript_read".to_string(),
                result_count: None,
                unavailable_reason: Some("auth token unavailable".to_string()),
                warnings: vec!["secret warning".to_string()],
            },
            failure_reason: Some("secret=abc123".to_string()),
            audit_refs: vec!["audit:attempt:secret".to_string()],
            warnings: vec!["token warning".to_string()],
        });
        let attention = vec![RuntimeSessionAttention {
            attention_id: "attention-secret".to_string(),
            project_id: Some("project-1".to_string()),
            workflow_id: Some("workflow-1".to_string()),
            node_id: Some("node-1".to_string()),
            session_id: Some("session-1".to_string()),
            adapter_id: "codex-local".to_string(),
            source_refs: vec![RuntimeAttentionSourceRef {
                source_kind: "session_continuation_attempt".to_string(),
                source_id: "attempt-secret".to_string(),
                label: "attempt".to_string(),
            }],
            kind: "readback_unavailable".to_string(),
            severity: "warning".to_string(),
            status: "readback_unavailable".to_string(),
            title: "readback unavailable".to_string(),
            user_message: "transcript unavailable with oauth token".to_string(),
            technical_summary: "raw provider credential blocked".to_string(),
            recommended_next_step: "inspect boundary".to_string(),
            requires_user_action: true,
            blocks_continuation: false,
            readback_boundary: ReadbackBoundaryStatus {
                status: "readback_unavailable".to_string(),
                reason: "not_attempted_stub".to_string(),
                attempted: false,
                real_readback_performed: false,
                result_count: None,
                user_message: "readback unavailable".to_string(),
                technical_summary: "token blocked".to_string(),
                source_refs: vec![],
                warnings: vec![],
            },
            created_at: "2026-06-07T00:00:02Z".to_string(),
            updated_at: "2026-06-07T00:00:02Z".to_string(),
            warnings: vec![],
        }];

        let runtime_store = derive_store_from_runtime_state(
            "/tmp/workflow-state.v0.json",
            &store,
            &attention,
            "2026-06-07T00:00:03Z",
        );
        let serialized = serde_json::to_string(&runtime_store).expect("serialize runtime log");

        for expected_category in [
            "app_session",
            "workflow_run",
            "dispatch_attempt",
            "readback",
            "permission_wait",
            "diagnostic_event",
        ] {
            assert!(
                runtime_store
                    .entries
                    .iter()
                    .any(|entry| entry.category == expected_category),
                "missing category {expected_category}"
            );
        }
        assert!(runtime_store
            .boundary
            .separation_rule
            .contains("不能互相替代"));
        assert!(runtime_store
            .entries
            .iter()
            .all(|entry| entry.redaction_status == "redacted_safe_summary"));
        assert!(runtime_store
            .entries
            .iter()
            .any(|entry| !entry.audit_refs.is_empty()));
        for forbidden in [
            "sk-test-secret",
            "abc123",
            "oauth",
            "raw provider credential",
            "full transcript",
            "auth token",
            "secret=abc123",
        ] {
            assert!(
                !serialized.to_lowercase().contains(forbidden),
                "runtime log leaked forbidden fragment {forbidden}"
            );
        }
    }

    fn empty_continuation_store() -> SessionContinuationStoreV1 {
        SessionContinuationStoreV1 {
            schema_version: "session_continuation_store.v1".to_string(),
            store_version: 1,
            storage_kind: "sidecar_json_v0".to_string(),
            scope: SessionContinuationStoreScope {
                scope_kind: "workflow_state_sidecar".to_string(),
                workflow_state_path: Some("/tmp/workflow-state.v0.json".to_string()),
                sidecar_path: Some("/tmp/session-continuations.v1.json".to_string()),
                project_roots: vec!["/tmp/project".to_string()],
            },
            revision: 0,
            last_write_id: None,
            generated_by: "test".to_string(),
            created_at: "2026-06-07T00:00:00Z".to_string(),
            updated_at: "2026-06-07T00:00:00Z".to_string(),
            continuations: vec![],
            attempts: vec![],
            audit_events: vec![],
            warnings: vec![],
        }
    }
}
