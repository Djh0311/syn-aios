use crate::{
    FormalMemoryStoreV1, MemoryCandidateStoreV1, MemoryCaptureStoreV1, ObservationStatus,
    ObservationStoreV1, RealExecutionProductCommandStore, RuntimeLogStoreV1,
    SessionContinuationStoreV1, StoreIntegrityFinding,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const BOUNDARY: &str =
    "Stage K K2.5 只读跨 sidecar 一致性扫描；只生成 finding，不修复、不迁移、不写正式记忆。";

pub(crate) fn derive_store_integrity_findings(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Vec<StoreIntegrityFinding> {
    let mut findings = Vec::new();

    let product_store =
        match crate::real_execution_command::load_real_execution_product_command_store(
            workflow_state_path,
            timestamp,
        ) {
            Ok((store, available, path)) => Some((store, available, path)),
            Err(error) => {
                findings.push(load_error_finding(
                    "product_command_consistency",
                    "Product Command 链路一致性",
                    path_display(
                        crate::real_execution_command::real_execution_product_command_sidecar_path(
                            workflow_state_path,
                        ),
                    ),
                    error,
                ));
                None
            }
        };
    let continuation_store =
        match crate::session_continuation_store::load_store(workflow_state_path, timestamp) {
            Ok(store) => Some(store),
            Err(error) => {
                findings.push(load_error_finding(
                    "session_continuation_consistency",
                    "会话 continuation 链路一致性",
                    path_display(crate::session_continuation_store::sidecar_path(
                        workflow_state_path,
                    )),
                    error,
                ));
                None
            }
        };
    let runtime_store = match crate::runtime_log_store::load_store(workflow_state_path) {
        Ok(store) => Some(store),
        Err(error) if error == "runtime_log_sidecar_missing" => None,
        Err(error) => {
            findings.push(load_error_finding(
                "runtime_log_consistency",
                "Runtime log 链路一致性",
                path_display(crate::runtime_log_store::sidecar_path(workflow_state_path)),
                error,
            ));
            None
        }
    };
    let capture_store = match crate::memory_capture_bus::load_store(workflow_state_path, timestamp)
    {
        Ok(store) => Some(store),
        Err(error) => {
            findings.push(load_error_finding(
                "memory_capture_consistency",
                "记忆 capture 链路一致性",
                path_display(crate::memory_capture_bus::sidecar_path(workflow_state_path)),
                error,
            ));
            None
        }
    };
    let observation_store =
        match crate::observation_store::load_store(workflow_state_path, timestamp) {
            Ok(store) => Some(store),
            Err(error) => {
                findings.push(load_error_finding(
                    "observation_consistency",
                    "Observation 链路一致性",
                    path_display(crate::observation_store::sidecar_path(workflow_state_path)),
                    error,
                ));
                None
            }
        };
    let candidate_store =
        match crate::memory_candidate_store::load_store(workflow_state_path, timestamp) {
            Ok(store) => Some(store),
            Err(error) => {
                findings.push(load_error_finding(
                    "memory_candidate_consistency",
                    "候选记忆链路一致性",
                    path_display(crate::memory_candidate_store::sidecar_path(
                        workflow_state_path,
                    )),
                    error,
                ));
                None
            }
        };
    let formal_store = match crate::formal_memory_store::load_store(workflow_state_path, timestamp)
    {
        Ok(store) => Some(store),
        Err(error) => {
            findings.push(load_error_finding(
                "formal_memory_consistency",
                "正式记忆链路一致性",
                path_display(crate::formal_memory_store::sidecar_path(
                    workflow_state_path,
                )),
                error,
            ));
            None
        }
    };

    if let (Some((product_store, product_available, product_path)), Some(continuation_store)) =
        (&product_store, &continuation_store)
    {
        append_execution_findings(
            &mut findings,
            product_store,
            *product_available,
            product_path,
            continuation_store,
            runtime_store.as_ref(),
        );
    }
    if let (
        Some(capture_store),
        Some(observation_store),
        Some(candidate_store),
        Some(formal_store),
    ) = (
        &capture_store,
        &observation_store,
        &candidate_store,
        &formal_store,
    ) {
        append_memory_findings(
            &mut findings,
            capture_store,
            observation_store,
            candidate_store,
            formal_store,
            product_store.as_ref().map(|(store, _, _)| store),
            runtime_store.as_ref(),
        );
    }

    if findings.is_empty() {
        findings.push(StoreIntegrityFinding {
            store_id: "cross_sidecar_consistency".to_string(),
            label: "跨 sidecar 一致性".to_string(),
            status: "ok".to_string(),
            severity: "info".to_string(),
            path: workflow_state_path
                .parent()
                .map(|parent| parent.display().to_string()),
            schema_version: Some("stage_k_k2_5_consistency_scan.v1".to_string()),
            revision: None,
            item_count: 0,
            warning_count: 0,
            error: None,
            summary: "Product Command / Continuation / RuntimeLog / Capture / Observation / Candidate / FormalMemory 未发现缺链。".to_string(),
            boundary: BOUNDARY.to_string(),
        });
    }

    findings
}

fn append_execution_findings(
    findings: &mut Vec<StoreIntegrityFinding>,
    product_store: &RealExecutionProductCommandStore,
    product_available: bool,
    product_path: &Path,
    continuation_store: &SessionContinuationStoreV1,
    runtime_store: Option<&RuntimeLogStoreV1>,
) {
    let continuation_ids = continuation_store
        .continuations
        .iter()
        .map(|continuation| continuation.continuation_id.as_str())
        .collect::<BTreeSet<_>>();
    let continuation_attempt_ids = continuation_store
        .attempts
        .iter()
        .map(|attempt| attempt.attempt_id.as_str())
        .collect::<BTreeSet<_>>();

    for attempt in &product_store.attempts {
        if let Some(continuation_id) = &attempt.continuation_id {
            if !continuation_ids.contains(continuation_id.as_str()) {
                findings.push(warning_finding(
                    "product_attempt_missing_continuation",
                    &attempt.attempt_id,
                    "Product Command attempt 指向的 continuation 不存在；不能把该 attempt 解释成完整会话执行链路。",
                    Some(product_path.display().to_string()),
                ));
            }
        }
        if let Some(runtime_ref) = &attempt.runtime_log_ref {
            if !runtime_ref_exists(runtime_store, runtime_ref) {
                findings.push(warning_finding(
                    "product_attempt_missing_runtime_log",
                    &attempt.attempt_id,
                    "Product Command attempt 指向的 runtime log ref 不存在或 runtime log sidecar 不可用。",
                    Some(product_path.display().to_string()),
                ));
            }
        } else if product_available
            && (attempt.runner_call_allowed || attempt.prompt_sent || attempt.real_codex_executed)
        {
            findings.push(warning_finding(
                "product_attempt_missing_runtime_ref",
                &attempt.attempt_id,
                "真实或准真实 Product Command attempt 缺少 runtime_log_ref。",
                Some(product_path.display().to_string()),
            ));
        }
        append_readback_count_finding(
            findings,
            "product_attempt_readback_count_not_null",
            &attempt.attempt_id,
            &attempt.readback_summary.status,
            attempt.readback_summary.result_count,
            Some(product_path.display().to_string()),
        );
    }

    for attempt in &continuation_store.attempts {
        if !continuation_ids.contains(attempt.continuation_id.as_str()) {
            findings.push(warning_finding(
                "continuation_attempt_orphan",
                &attempt.attempt_id,
                "Continuation attempt 指向的 continuation 不存在。",
                None,
            ));
        }
        if runtime_store.is_none() {
            findings.push(warning_finding(
                "continuation_attempt_missing_runtime_store",
                &attempt.attempt_id,
                "Continuation attempt 存在，但 runtime log sidecar 缺失；运行摘要不能声称完整。",
                None,
            ));
        } else if !runtime_attempt_ref_exists(runtime_store, &attempt.attempt_id) {
            findings.push(warning_finding(
                "continuation_attempt_missing_runtime_entry",
                &attempt.attempt_id,
                "Continuation attempt 没有对应 runtime log entry。",
                None,
            ));
        }
        append_readback_count_finding(
            findings,
            "continuation_attempt_readback_count_not_null",
            &attempt.attempt_id,
            &attempt.readback_summary.status,
            attempt.readback_summary.result_count,
            None,
        );
    }

    if let Some(runtime_store) = runtime_store {
        for entry in &runtime_store.entries {
            for source in &entry.source_refs {
                if source.source_kind == "controlled_session_continuation"
                    && !continuation_ids.contains(source.source_id.as_str())
                {
                    findings.push(warning_finding(
                        "runtime_entry_orphan_continuation",
                        &entry.entry_id,
                        "Runtime log entry 指向不存在的 continuation。",
                        None,
                    ));
                }
                if source.source_kind == "session_continuation_attempt"
                    && !continuation_attempt_ids.contains(source.source_id.as_str())
                {
                    findings.push(warning_finding(
                        "runtime_entry_orphan_attempt",
                        &entry.entry_id,
                        "Runtime log entry 指向不存在的 continuation attempt。",
                        None,
                    ));
                }
            }
        }
    }
}

fn append_memory_findings(
    findings: &mut Vec<StoreIntegrityFinding>,
    capture_store: &MemoryCaptureStoreV1,
    observation_store: &ObservationStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    formal_store: &FormalMemoryStoreV1,
    product_store: Option<&RealExecutionProductCommandStore>,
    runtime_store: Option<&RuntimeLogStoreV1>,
) {
    let observation_ids = observation_store
        .observations
        .iter()
        .map(|observation| observation.observation_id.as_str())
        .collect::<BTreeSet<_>>();
    let candidate_keys = candidate_store
        .candidates
        .iter()
        .map(|candidate| candidate.candidate_key.as_str())
        .collect::<BTreeSet<_>>();
    let formal_memory_ids = formal_store
        .records
        .iter()
        .map(|record| record.memory_id.as_str())
        .collect::<BTreeSet<_>>();
    let formal_version_ids = formal_store
        .versions
        .iter()
        .map(|version| version.version_id.as_str())
        .collect::<BTreeSet<_>>();
    let formal_audit_ids = formal_store
        .audit_events
        .iter()
        .map(|event| event.audit_event_id.as_str())
        .collect::<BTreeSet<_>>();
    let product_command_ids = product_store
        .map(|store| {
            store
                .commands
                .iter()
                .map(|command| command.product_command_id.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let product_attempt_ids = product_store
        .map(|store| {
            store
                .attempts
                .iter()
                .map(|attempt| attempt.attempt_id.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    for event in &capture_store.events {
        if matches!(
            event.candidate_policy.as_str(),
            "observation_only" | "candidate_allowed"
        ) && event.observation_id.is_none()
        {
            findings.push(warning_finding(
                "capture_event_missing_observation_id",
                &event.capture_event_id,
                "Capture event 策略要求 observation，但缺少 observation_id。",
                None,
            ));
        }
        if let Some(observation_id) = &event.observation_id {
            if !observation_ids.contains(observation_id.as_str()) {
                findings.push(warning_finding(
                    "capture_event_missing_observation",
                    &event.capture_event_id,
                    "Capture event 指向的 observation 不存在。",
                    None,
                ));
            }
        }
        if event.candidate_policy == "candidate_allowed" && event.candidate_key.is_none() {
            findings.push(warning_finding(
                "capture_event_missing_candidate_key",
                &event.capture_event_id,
                "Capture event 策略允许 candidate，但缺少 candidate_key。",
                None,
            ));
        }
        if let Some(candidate_key) = &event.candidate_key {
            if !candidate_keys.contains(candidate_key.as_str()) {
                findings.push(warning_finding(
                    "capture_event_missing_candidate",
                    &event.capture_event_id,
                    "Capture event 指向的 memory candidate 不存在。",
                    None,
                ));
            }
        }
        if let Some(product_command_id) = &event.product_command_id {
            if product_store.is_some() && !product_command_ids.contains(product_command_id.as_str())
            {
                findings.push(warning_finding(
                    "capture_event_missing_product_command",
                    &event.capture_event_id,
                    "Capture event 指向的 Product Command 不存在。",
                    None,
                ));
            }
        }
        if let Some(product_attempt_id) = &event.product_attempt_id {
            if product_store.is_some() && !product_attempt_ids.contains(product_attempt_id.as_str())
            {
                findings.push(warning_finding(
                    "capture_event_missing_product_attempt",
                    &event.capture_event_id,
                    "Capture event 指向的 Product Command attempt 不存在。",
                    None,
                ));
            }
        }
        if let Some(runtime_log_ref) = &event.runtime_log_ref {
            if !runtime_ref_exists(runtime_store, runtime_log_ref) {
                findings.push(warning_finding(
                    "capture_event_missing_runtime_log",
                    &event.capture_event_id,
                    "Capture event 指向的 runtime log ref 不存在或 runtime log sidecar 不可用。",
                    None,
                ));
            }
        }
    }

    for observation in &observation_store.observations {
        if observation.status == ObservationStatus::CandidateCreated
            && observation.candidate_key.is_none()
        {
            findings.push(warning_finding(
                "observation_candidate_status_missing_key",
                &observation.observation_id,
                "Observation 状态为 candidate_created，但缺少 candidate_key。",
                None,
            ));
        }
        if let Some(candidate_key) = &observation.candidate_key {
            if !candidate_keys.contains(candidate_key.as_str()) {
                findings.push(warning_finding(
                    "observation_missing_candidate",
                    &observation.observation_id,
                    "Observation 指向的 memory candidate 不存在。",
                    None,
                ));
            }
        }
    }

    for candidate in &candidate_store.candidates {
        if let Some(observation_id) = candidate.generated_from.strip_prefix("observation:") {
            if !observation_ids.contains(observation_id) {
                findings.push(warning_finding(
                    "candidate_missing_source_observation",
                    &candidate.candidate_key,
                    "Memory candidate 的 generated_from observation 不存在。",
                    None,
                ));
            }
            if let Some(observation) = observation_store
                .observations
                .iter()
                .find(|item| item.observation_id == observation_id)
            {
                if observation.candidate_key.as_deref() != Some(candidate.candidate_key.as_str()) {
                    findings.push(warning_finding(
                        "candidate_observation_backlink_missing",
                        &candidate.candidate_key,
                        "Memory candidate 的来源 observation 没有回链到该 candidate_key。",
                        None,
                    ));
                }
            }
        }
        for source in &candidate.source_refs {
            if source.source_type == "observation_ref" {
                if let Some(source_id) = &source.source_id {
                    if !observation_ids.contains(source_id.as_str()) {
                        findings.push(warning_finding(
                            "candidate_source_observation_missing",
                            &candidate.candidate_key,
                            "Memory candidate source_ref 指向的 observation 不存在。",
                            None,
                        ));
                    }
                }
            }
        }
        if let Some(adoption) = &candidate.adoption {
            if !formal_memory_ids.contains(adoption.adopted_memory_id.as_str()) {
                findings.push(warning_finding(
                    "candidate_adoption_missing_formal_memory",
                    &candidate.candidate_key,
                    "Memory candidate adoption 指向的正式记忆不存在。",
                    None,
                ));
            }
            if !formal_version_ids.contains(adoption.adopted_version_id.as_str()) {
                findings.push(warning_finding(
                    "candidate_adoption_missing_formal_version",
                    &candidate.candidate_key,
                    "Memory candidate adoption 指向的正式记忆版本不存在。",
                    None,
                ));
            }
            if !formal_audit_ids.contains(adoption.adopted_audit_event_id.as_str()) {
                findings.push(warning_finding(
                    "candidate_adoption_missing_formal_audit",
                    &candidate.candidate_key,
                    "Memory candidate adoption 指向的正式记忆审计事件不存在。",
                    None,
                ));
            }
        }
    }

    for record in &formal_store.records {
        for source in &record.source_refs {
            if source.source_type == "observation_ref" {
                if let Some(source_id) = &source.source_id {
                    if !observation_ids.contains(source_id.as_str()) {
                        findings.push(warning_finding(
                            "formal_memory_source_observation_missing",
                            &record.memory_id,
                            "FormalMemory source_ref 指向的 observation 不存在。",
                            None,
                        ));
                    }
                }
            }
        }
    }
    for audit_event in &formal_store.audit_events {
        if audit_event.event_type == "memory_candidate_adopted_to_formal_memory" {
            if let Some(candidate_key) = candidate_key_from_reason(&audit_event.reason) {
                let matching_candidate = candidate_store
                    .candidates
                    .iter()
                    .find(|candidate| candidate.candidate_key == candidate_key);
                match matching_candidate {
                    Some(candidate) => {
                        let adopted_memory_id = audit_event.target_id.as_deref();
                        if candidate
                            .adoption
                            .as_ref()
                            .map(|adoption| adoption.adopted_memory_id.as_str())
                            != adopted_memory_id
                        {
                            findings.push(warning_finding(
                                "formal_memory_missing_candidate_adoption_link",
                                &audit_event.audit_event_id,
                                "正式记忆审计显示来自 candidate 采纳，但 candidate 缺少匹配 adoption 回链。",
                                None,
                            ));
                        }
                    }
                    None => findings.push(warning_finding(
                        "formal_memory_missing_source_candidate",
                        &audit_event.audit_event_id,
                        "正式记忆审计显示来自 candidate 采纳，但 candidate 不存在。",
                        None,
                    )),
                }
            }
        }
    }
}

fn append_readback_count_finding(
    findings: &mut Vec<StoreIntegrityFinding>,
    kind: &str,
    target_id: &str,
    status: &str,
    result_count: Option<i64>,
    path: Option<String>,
) {
    if readback_requires_null_count(status) && result_count.is_some() {
        findings.push(warning_finding(
            kind,
            target_id,
            "readback unavailable / failed / timed_out 不能显示为真实 0 条或具体条数。",
            path,
        ));
    }
}

fn readback_requires_null_count(status: &str) -> bool {
    matches!(
        status,
        "readback_unavailable" | "readback_failed" | "readback_timed_out"
    )
}

fn runtime_ref_exists(runtime_store: Option<&RuntimeLogStoreV1>, runtime_ref: &str) -> bool {
    runtime_store.is_some_and(|store| {
        store.entries.iter().any(|entry| {
            entry.entry_id == runtime_ref
                || entry
                    .source_refs
                    .iter()
                    .any(|source| source.source_id == runtime_ref)
        })
    })
}

fn runtime_attempt_ref_exists(runtime_store: Option<&RuntimeLogStoreV1>, attempt_id: &str) -> bool {
    runtime_store.is_some_and(|store| {
        store.entries.iter().any(|entry| {
            entry.source_refs.iter().any(|source| {
                source.source_kind == "session_continuation_attempt"
                    && source.source_id == attempt_id
            })
        })
    })
}

fn candidate_key_from_reason(reason: &str) -> Option<String> {
    let marker = "candidate_key=";
    let (_, rest) = reason.split_once(marker)?;
    let value = rest
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ';' | '；' | ',' | '，'))
        .next()
        .unwrap_or("")
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn load_error_finding(
    store_id: &str,
    label: &str,
    path: Option<String>,
    error: String,
) -> StoreIntegrityFinding {
    StoreIntegrityFinding {
        store_id: store_id.to_string(),
        label: label.to_string(),
        status: "degraded".to_string(),
        severity: "degraded".to_string(),
        path,
        schema_version: Some("stage_k_k2_5_consistency_scan.v1".to_string()),
        revision: None,
        item_count: 0,
        warning_count: 1,
        error: Some(error),
        summary: format!("{label} 无法读取，K2.5 只报告不覆盖。"),
        boundary: BOUNDARY.to_string(),
    }
}

fn warning_finding(
    kind: &str,
    target_id: &str,
    summary: &str,
    path: Option<String>,
) -> StoreIntegrityFinding {
    StoreIntegrityFinding {
        store_id: format!(
            "cross_sidecar_consistency:{}:{}",
            kind,
            compact_id(target_id)
        ),
        label: "跨 sidecar 一致性".to_string(),
        status: "warning".to_string(),
        severity: "warning".to_string(),
        path,
        schema_version: Some("stage_k_k2_5_consistency_scan.v1".to_string()),
        revision: None,
        item_count: 1,
        warning_count: 1,
        error: Some(kind.to_string()),
        summary: summary.to_string(),
        boundary: BOUNDARY.to_string(),
    }
}

fn compact_id(value: &str) -> String {
    let compact = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
        .take(72)
        .collect::<String>();
    if compact.is_empty() {
        "unknown".to_string()
    } else {
        compact
    }
}

fn path_display(path: Result<PathBuf, String>) -> Option<String> {
    path.ok().map(|path| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn memory_consistency_reports_capture_missing_downstream_links() {
        let workflow_state_path = isolated_workflow_state_path("capture-missing-links");
        let sidecar = crate::memory_capture_bus::sidecar_path(&workflow_state_path).unwrap();
        fs::write(
            &sidecar,
            serde_json::to_string_pretty(&json!({
                "store_version": "memory_capture_store.v1",
                "project_id": null,
                "workflow_id": null,
                "revision": 1,
                "events": [{
                    "capture_event_id": "memory-capture:test:missing-links",
                    "event_key": "event-key",
                    "schema_version": "memory_capture_event.v1",
                    "source_type": "worker_report",
                    "source_ref_id": "source:test",
                    "project_id": null,
                    "workflow_id": null,
                    "workflow_node_id": null,
                    "run_unit_id": null,
                    "product_command_id": null,
                    "product_attempt_id": null,
                    "runtime_log_ref": null,
                    "audit_refs": [],
                    "readback_ref": null,
                    "task_package_ref": null,
                    "memory_packet_ref": null,
                    "summary": "capture summary",
                    "evidence_summary": "evidence summary",
                    "sensitivity": "internal",
                    "candidate_policy": "candidate_allowed",
                    "blocked_reason": null,
                    "observation_id": "obs:missing",
                    "candidate_key": "candidate:missing",
                    "created_by": "test",
                    "created_at": "2026-06-10T00:00:00Z",
                    "updated_at": "2026-06-10T00:00:00Z"
                }],
                "updated_at": "2026-06-10T00:00:00Z",
                "warnings": []
            }))
            .unwrap(),
        )
        .unwrap();

        let findings =
            derive_store_integrity_findings(&workflow_state_path, "2026-06-10T00:00:01Z");
        assert!(findings
            .iter()
            .any(|finding| finding.error.as_deref() == Some("capture_event_missing_observation")));
        assert!(findings
            .iter()
            .any(|finding| finding.error.as_deref() == Some("capture_event_missing_candidate")));
    }

    fn isolated_workflow_state_path(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "workbench-memory-consistency-{label}-{}-{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.join("workflow-state.v0.json")
    }
}
