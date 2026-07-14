use super::{load_store, session, workflow_state_path, McpServerConfig, SupervisorWorker};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReviewEvidenceField {
    ByteCount,
    TrailingNewline,
    Sha256,
}

impl ReviewEvidenceField {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::ByteCount => "byte_count",
            Self::TrailingNewline => "trailing_newline",
            Self::Sha256 => "sha256",
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct ReviewEvidenceGateOutcome {
    pub(super) required_fields: Vec<ReviewEvidenceField>,
    pub(super) readonly_reviewer_worker_ids: Vec<String>,
    pub(super) advisories: Vec<String>,
}

/// 把授权 checks 中确实需要文件实证的少数语义识别出来。
///
/// 这是保守白名单，而不是把所有文字验收都变成终标硬闸：只有字节/大小、换行或哈希
/// 标准要求 `review_evidence`；编译、测试、内容判断等仍由主管 advisory 判断。
pub(super) fn authorization_requires_review_evidence(allowed_checks: &[String]) -> bool {
    !crate::supervisor_session_launcher::supervisor_pilot_byte_family_checks(allowed_checks)
        .is_empty()
}

fn required_review_evidence_fields(allowed_checks: &[String]) -> Vec<ReviewEvidenceField> {
    let mut required = Vec::new();
    for check in
        crate::supervisor_session_launcher::supervisor_pilot_byte_family_checks(allowed_checks)
    {
        let mut classified = false;
        if check_requires_byte_count(&check) && !required.contains(&ReviewEvidenceField::ByteCount)
        {
            required.push(ReviewEvidenceField::ByteCount);
            classified = true;
        }
        if check_requires_trailing_newline(&check)
            && !required.contains(&ReviewEvidenceField::TrailingNewline)
        {
            required.push(ReviewEvidenceField::TrailingNewline);
            classified = true;
        }
        if check_requires_sha256(&check) && !required.contains(&ReviewEvidenceField::Sha256) {
            required.push(ReviewEvidenceField::Sha256);
            classified = true;
        }
        // launcher 的同一白名单已决定要物化 reviewer；若它命中的是未来新增的字节族措辞，
        // 这里保守要求 byte_count，不能出现“已派 reviewer 而终标无闸”的漂移。
        if !classified && !required.contains(&ReviewEvidenceField::ByteCount) {
            required.push(ReviewEvidenceField::ByteCount);
        }
    }
    required
}

fn check_requires_byte_count(check: &str) -> bool {
    let normalized = check.trim().to_ascii_lowercase();
    ["byte", "byte_count", "file size"]
        .iter()
        .any(|token| normalized.contains(token))
        || ["字节", "大小", "尺寸"]
            .iter()
            .any(|token| check.contains(token))
}

fn check_requires_trailing_newline(check: &str) -> bool {
    let normalized = check.trim().to_ascii_lowercase();
    ["newline", "trailing_newline"]
        .iter()
        .any(|token| normalized.contains(token))
        || check.contains("换行")
}

fn check_requires_sha256(check: &str) -> bool {
    let normalized = check.trim().to_ascii_lowercase();
    ["sha256", "sha-256", "checksum", "hash"]
        .iter()
        .any(|token| normalized.contains(token))
        || ["哈希", "散列", "校验和"]
            .iter()
            .any(|token| check.contains(token))
}

pub(super) fn review_evidence_gate_for_pass(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
    authorization: &crate::PlanAuthorization,
) -> Result<Option<ReviewEvidenceGateOutcome>, String> {
    // 站 3b 的零写单沿用原 finalize 行为；没有白名单文件标准的写单也不因本包受阻。
    if authorization.scope.allowed_write_roots.is_empty() {
        return Ok(None);
    }
    let required_fields = required_review_evidence_fields(&authorization.scope.allowed_checks);
    if required_fields.is_empty() {
        return Ok(None);
    }

    let workflow_state = crate::read_workflow_state_value(workflow_state_path(config)?)?;
    let store = load_store(config)?;
    let session = session(&store, &config.run_id).ok_or_else(|| {
        "final_mark 拒绝 pass：当前 run 未找到主管 worker 账本，无法核验只读复核实证。".to_string()
    })?;
    let mut readonly_reviewer_worker_ids = Vec::new();
    let mut evidence = Vec::new();
    for worker in session.workers.iter().filter(|worker| {
        worker.project_root == project_root
            && worker.workflow_id == workflow_id
            && worker.authorization_id == authorization_id
            && worker.allowed_write.is_empty()
            && is_exact_station4_reviewer_worker(
                &workflow_state,
                worker,
                project_root,
                workflow_id,
                authorization_id,
            )
    }) {
        let Some(report) = worker.last_report.as_ref() else {
            continue;
        };
        let Ok(worker_evidence) =
            serde_json::from_value::<Vec<crate::worker_report::WorkerReviewEvidence>>(
                report
                    .get("review_evidence")
                    .cloned()
                    .unwrap_or(Value::Null),
            )
        else {
            continue;
        };
        if worker_evidence.is_empty() {
            continue;
        }
        readonly_reviewer_worker_ids.push(worker.worker_id.clone());
        evidence.extend(worker_evidence);
    }

    if evidence.is_empty() {
        return Err(format!(
            "final_mark 拒绝 pass：本 run 缺少与当前授权绑定的 Station4 只读复核 worker 的结构化实证块；授权要求 {}。请派发授权派生的 allowed_write=[] 复核任务，并在回程 review_evidence 数组补 path、byte_count、sha256、trailing_newline、read_method。",
            relevant_review_checks(&authorization.scope.allowed_checks)
        ));
    }

    let missing = required_fields
        .iter()
        .copied()
        .filter(|field| !review_evidence_covers(*field, &evidence))
        .map(ReviewEvidenceField::key)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "final_mark 拒绝 pass：只读复核实证未覆盖授权的字节级标准，缺 {}；相关 checks：{}。请补同一类机器可核字段后再终标。",
            missing.join("、"),
            relevant_review_checks(&authorization.scope.allowed_checks)
        ));
    }

    Ok(Some(ReviewEvidenceGateOutcome {
        required_fields,
        readonly_reviewer_worker_ids,
        advisories: review_evidence_advisories(&authorization.scope.allowed_checks, &evidence),
    }))
}

fn is_exact_station4_reviewer_worker(
    workflow_state: &Value,
    worker: &SupervisorWorker,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
) -> bool {
    let expected_planned_task_id =
        crate::supervisor_session_launcher::supervisor_pilot_readonly_reviewer_task_id(
            authorization_id,
        );
    let marker = crate::supervisor_session_launcher::SUPERVISOR_PILOT_READONLY_BYTE_REVIEW_MARKER;
    let expected_project_id = crate::project_id(project_root);
    workflow_state
        .get("artifacts")
        .and_then(Value::as_array)
        .is_some_and(|artifacts| {
            artifacts.iter().any(|artifact| {
                exact_artifact_string(artifact, "artifact_type", "task_package")
                    && exact_artifact_string(artifact, "source_ref", &worker.work_item_id)
                    && exact_artifact_string(artifact, "project_id", &expected_project_id)
                    && exact_artifact_string(artifact, "workflow_id", workflow_id)
                    && exact_artifact_string(
                        artifact,
                        "project_director_planned_task_id",
                        &expected_planned_task_id,
                    )
                    && exact_artifact_string(artifact, "title", "只读复核：站4字节级实证")
                    && exact_artifact_string(artifact, "task_name", "只读复核：站4字节级实证")
                    && artifact
                        .get("task_goal")
                        .and_then(Value::as_str)
                        .is_some_and(|task_goal| task_goal.contains(marker))
                    && artifact
                        .get("allowed_write")
                        .and_then(Value::as_array)
                        .is_some_and(|roots| roots.is_empty())
                    && artifact
                        .get("forbidden_actions")
                        .and_then(Value::as_array)
                        .is_some_and(|actions| {
                            actions.iter().any(|action| action.as_str() == Some(marker))
                        })
            })
        })
}

fn exact_artifact_string(artifact: &Value, key: &str, expected: &str) -> bool {
    crate::optional_string_from(artifact, key).as_deref() == Some(expected)
}

fn review_evidence_covers(
    field: ReviewEvidenceField,
    evidence: &[crate::worker_report::WorkerReviewEvidence],
) -> bool {
    evidence.iter().any(|entry| {
        !entry.path.trim().is_empty()
            && !entry.read_method.trim().is_empty()
            && match field {
                ReviewEvidenceField::ByteCount => entry.byte_count.is_some(),
                ReviewEvidenceField::TrailingNewline => entry.trailing_newline.is_some(),
                ReviewEvidenceField::Sha256 => is_sha256(&entry.sha256),
            }
    })
}

fn relevant_review_checks(allowed_checks: &[String]) -> String {
    let byte_family_checks =
        crate::supervisor_session_launcher::supervisor_pilot_byte_family_checks(allowed_checks);
    let checks = byte_family_checks
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if checks.is_empty() {
        "（未列出）".to_string()
    } else {
        checks.join("；")
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

fn review_evidence_advisories(
    allowed_checks: &[String],
    evidence: &[crate::worker_report::WorkerReviewEvidence],
) -> Vec<String> {
    let mut advisories = Vec::new();
    for check in
        crate::supervisor_session_launcher::supervisor_pilot_byte_family_checks(allowed_checks)
    {
        if check_requires_byte_count(&check) {
            if let Some(expected) =
                expected_integer_near_marker(&check, &["字节", "byte", "bytes", "size"])
            {
                for entry in evidence.iter().filter_map(|entry| {
                    entry.byte_count.map(|actual| (entry.path.as_str(), actual))
                }) {
                    if entry.1 != expected {
                        advisories.push(format!(
                            "复核实证 {} 的 byte_count={} 与授权 check「{}」声明的 {} 不一致；已如实呈现，仍由主管决定 verdict。",
                            display_review_path(entry.0), entry.1, check, expected
                        ));
                    }
                }
            }
        }
        if check_requires_trailing_newline(&check) {
            if let Some(expected) = expected_trailing_newline(&check) {
                for entry in evidence.iter().filter_map(|entry| {
                    entry
                        .trailing_newline
                        .map(|actual| (entry.path.as_str(), actual))
                }) {
                    if entry.1 != expected {
                        advisories.push(format!(
                            "复核实证 {} 的 trailing_newline={} 与授权 check「{}」不一致；已如实呈现，仍由主管决定 verdict。",
                            display_review_path(entry.0), entry.1, check
                        ));
                    }
                }
            }
        }
        if check_requires_sha256(&check) {
            if let Some(expected) = expected_sha256(&check) {
                for entry in evidence.iter().filter(|entry| is_sha256(&entry.sha256)) {
                    if !entry.sha256.eq_ignore_ascii_case(&expected) {
                        advisories.push(format!(
                            "复核实证 {} 的 sha256={} 与授权 check「{}」不一致；已如实呈现，仍由主管决定 verdict。",
                            display_review_path(&entry.path), entry.sha256, check
                        ));
                    }
                }
            }
        }
    }
    advisories
}

fn display_review_path(path: &str) -> &str {
    if path.trim().is_empty() {
        "（未报路径）"
    } else {
        path
    }
}

fn expected_integer_near_marker(check: &str, markers: &[&str]) -> Option<u64> {
    let lowered = check.to_ascii_lowercase();
    for marker in markers {
        let marker = marker.to_ascii_lowercase();
        let mut start = 0;
        while let Some(relative) = lowered[start..].find(&marker) {
            let index = start + relative;
            if let Some(value) = last_ascii_integer(&lowered[..index]) {
                return Some(value);
            }
            if let Some(value) = first_ascii_integer(&lowered[index + marker.len()..]) {
                return Some(value);
            }
            start = index + marker.len();
        }
    }
    None
}

fn last_ascii_integer(text: &str) -> Option<u64> {
    let bytes = text.as_bytes();
    let mut end = bytes.len();
    while end > 0 && !bytes[end - 1].is_ascii_digit() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && bytes[start - 1].is_ascii_digit() {
        start -= 1;
    }
    (start < end)
        .then(|| text[start..end].parse::<u64>().ok())
        .flatten()
}

fn first_ascii_integer(text: &str) -> Option<u64> {
    let bytes = text.as_bytes();
    let mut start = 0;
    while start < bytes.len() && !bytes[start].is_ascii_digit() {
        start += 1;
    }
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    (start < end)
        .then(|| text[start..end].parse::<u64>().ok())
        .flatten()
}

fn expected_trailing_newline(check: &str) -> Option<bool> {
    let lowered = check.to_ascii_lowercase();
    let is_newline_check = check_requires_trailing_newline(&lowered);
    if !is_newline_check {
        return None;
    }
    if lowered.contains("trailing_newline=false")
        || lowered.contains("no trailing newline")
        || lowered.contains("without newline")
        || lowered.contains("无换行")
        || lowered.contains("不带换行")
        || lowered.contains("不得有换行")
    {
        return Some(false);
    }
    if lowered.contains("trailing_newline=true")
        || lowered.contains("with trailing newline")
        || lowered.contains("末尾有换行")
        || lowered.contains("带换行")
    {
        return Some(true);
    }
    None
}

fn expected_sha256(check: &str) -> Option<String> {
    if !check_requires_sha256(check) {
        return None;
    }
    check
        .split(|character: char| !character.is_ascii_hexdigit())
        .find(|candidate| candidate.len() == 64)
        .map(|candidate| candidate.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::{McpRole, McpServerConfig, SupervisorQuotaLimits};
    use serde_json::{json, Value};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const PROJECT: &str = "/Users/yoyi/codex-workflow-mario-test";
    const WORKFLOW: &str = "workflow:users-yoyi-codex-workflow-mario-test:default";
    const AUTH: &str = "plan-auth:station4-byte-review";
    const NODE: &str = "workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev";
    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct GateFixture {
        root: PathBuf,
        state_path: PathBuf,
        read_root: PathBuf,
        config: McpServerConfig,
    }

    impl GateFixture {
        fn new(allowed_write_roots: Vec<String>, allowed_checks: Vec<String>) -> Self {
            let unique = format!(
                "supervisor-review-evidence-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("system clock")
                    .as_nanos(),
                FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            );
            let root = std::env::temp_dir().join(unique);
            let read_root = root.join("read-root");
            fs::create_dir_all(&read_root).expect("fixture dirs");
            let state_path = root.join("workflow-state.json");
            fs::write(
                &state_path,
                json!({"workflow_chain_runs": [{"status": "running"}], "audit_events": []})
                    .to_string(),
            )
            .expect("workflow state");
            let fixture = Self {
                root,
                state_path: state_path.clone(),
                read_root,
                config: McpServerConfig {
                    role: McpRole::SupervisorOrchestrator,
                    run_id: "supervisor-review-evidence-test-run".to_string(),
                    node_id: None,
                    supervisor_workflow_state_path: Some(state_path),
                    supervisor_quota_limits: Some(SupervisorQuotaLimits {
                        max_active_workers: 2,
                        max_follow_ups_per_worker: 2,
                        max_runtime_minutes: 30,
                    }),
                },
            };
            fixture.write_active_authorization(allowed_write_roots, allowed_checks);
            super::super::record_pilot_session_started(
                &fixture.config,
                &super::super::SupervisorPilotSessionLaunch {
                    project_root: PROJECT.to_string(),
                    workflow_id: WORKFLOW.to_string(),
                    authorization_id: AUTH.to_string(),
                    model_id: "test-model".to_string(),
                    reasoning_effort: "medium".to_string(),
                    workbench_executable_path: "test-workbench".to_string(),
                    workbench_build_id: "test-build".to_string(),
                    supervisor_contract_version: "test-contract".to_string(),
                    supervisor_contract_sha256: "test-supervisor-sha".to_string(),
                    worker_report_contract_sha256: "test-worker-sha".to_string(),
                },
            )
            .expect("start supervisor session");
            fixture
        }

        fn write_active_authorization(
            &self,
            allowed_write_roots: Vec<String>,
            allowed_checks: Vec<String>,
        ) {
            let store = crate::PlanAuthorizationStoreV1 {
                schema_version: "plan_authorization_store.v1".to_string(),
                revision: 1,
                authorizations: vec![crate::PlanAuthorization {
                    authorization_id: AUTH.to_string(),
                    schema_version: "plan_authorization.v1".to_string(),
                    project_id: crate::project_id(PROJECT),
                    workflow_id: WORKFLOW.to_string(),
                    source_proposal_id: None,
                    title: "station4 byte review".to_string(),
                    goal_summary: "station4 byte review".to_string(),
                    status: crate::PlanAuthorizationStatus::Active,
                    scope: crate::AuthorizedExecutionScope {
                        project_id: crate::project_id(PROJECT),
                        workflow_id: WORKFLOW.to_string(),
                        allowed_role_ids: vec!["codex-dev".to_string()],
                        allowed_agent_ids: vec!["thread:station4-byte-review".to_string()],
                        allowed_read_roots: vec![self.read_root.display().to_string()],
                        allowed_write_roots,
                        allowed_tools: vec!["apply_patch".to_string()],
                        allowed_checks,
                        allowed_task_package_kinds: vec!["task_package".to_string()],
                        max_worker_dispatches: Some(2),
                        max_runtime_minutes: Some(30),
                        stop_conditions: vec![],
                    },
                    user_confirmation: None,
                    global_boundary_review: None,
                    audit_refs: vec![],
                    created_at_ms: super::super::now_ms(),
                    updated_at_ms: super::super::now_ms(),
                    expires_at_ms: None,
                }],
                audit_events: vec![],
                updated_at_ms: super::super::now_ms(),
                warnings: vec![],
            };
            let auth_path = crate::plan_authorization_store::sidecar_path(&self.state_path)
                .expect("authorization sidecar path");
            fs::write(
                auth_path,
                serde_json::to_vec(&store).expect("authorization json"),
            )
            .expect("write authorization store");
        }

        fn add_worker(
            &self,
            worker_id: &str,
            work_item_id: &str,
            allowed_write: Vec<String>,
            report: Value,
        ) {
            super::super::update_store(&self.config, "seed-review-evidence-worker", |store| {
                super::super::session_mut(store, &self.config.run_id)
                    .workers
                    .push(super::super::SupervisorWorker {
                        worker_id: worker_id.to_string(),
                        project_root: PROJECT.to_string(),
                        workflow_id: WORKFLOW.to_string(),
                        node_id: NODE.to_string(),
                        work_item_id: work_item_id.to_string(),
                        authorization_id: AUTH.to_string(),
                        native_thread_id: format!("thread:{worker_id}"),
                        dispatch_id: format!("dispatch:{worker_id}"),
                        allowed_write,
                        state: "completed".to_string(),
                        started_at_ms: super::super::now_ms(),
                        last_report: Some(report),
                        last_result_summary: "fixture review evidence".to_string(),
                        ..super::super::SupervisorWorker::default()
                    });
                Ok(())
            })
            .expect("seed review worker");
        }

        fn add_reviewer_task_artifact(
            &self,
            work_item_id: &str,
            planned_task_id: String,
            include_marker: bool,
        ) {
            let marker =
                crate::supervisor_session_launcher::SUPERVISOR_PILOT_READONLY_BYTE_REVIEW_MARKER;
            let mut state: Value =
                serde_json::from_slice(&fs::read(&self.state_path).expect("workflow state"))
                    .expect("workflow state json");
            let task_goal = if include_marker {
                format!("独立只读核对字节；{marker}")
            } else {
                "其他零写任务，不能冒充复核任务。".to_string()
            };
            let forbidden_actions = if include_marker {
                json!([marker])
            } else {
                json!(["其他零写约束"])
            };
            let artifact = json!({
                "artifact_type": "task_package",
                "source_ref": work_item_id,
                "project_id": crate::project_id(PROJECT),
                "workflow_id": WORKFLOW,
                "project_director_planned_task_id": planned_task_id,
                "title": "只读复核：站4字节级实证",
                "task_name": "只读复核：站4字节级实证",
                "task_goal": task_goal,
                "allowed_write": [],
                "forbidden_actions": forbidden_actions
            });
            if let Some(artifacts) = state.get_mut("artifacts").and_then(Value::as_array_mut) {
                artifacts.push(artifact);
            } else {
                state["artifacts"] = json!([artifact]);
            }
            fs::write(
                &self.state_path,
                serde_json::to_vec(&state).expect("workflow state json"),
            )
            .expect("write reviewer task artifact");
        }

        fn add_exact_reviewer(&self, worker_id: &str, work_item_id: &str, report: Value) {
            self.add_worker(worker_id, work_item_id, vec![], report);
            self.add_reviewer_task_artifact(
                work_item_id,
                crate::supervisor_session_launcher::supervisor_pilot_readonly_reviewer_task_id(
                    AUTH,
                ),
                true,
            );
        }

        fn finalize_pass(&self) -> Result<Value, String> {
            super::super::final_mark(
                &self.config,
                &json!({
                    "project_root": PROJECT,
                    "workflow_id": WORKFLOW,
                    "authorization_id": AUTH,
                    "verdict": "pass",
                    "reason": "fixture supervisor verdict"
                }),
            )
        }
    }

    impl Drop for GateFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn covered_review_evidence(byte_count: u64, trailing_newline: bool) -> Value {
        json!({
            "review_evidence": [{
                "path": "/p/output.txt",
                "byte_count": byte_count,
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "trailing_newline": trailing_newline,
                "read_method": "wc -c + sha256sum + tail"
            }]
        })
    }

    fn byte_and_newline_checks() -> Vec<String> {
        vec!["交付文件必须为 8 字节且末尾无换行".to_string()]
    }

    #[test]
    fn gate_uses_launcher_byte_family_classifier_only() {
        assert!(authorization_requires_review_evidence(
            &byte_and_newline_checks()
        ));
        assert!(authorization_requires_review_evidence(&[
            "output sha256 必须等于预期".to_string(),
        ]));
        assert!(!authorization_requires_review_evidence(&[
            "cargo test --lib".to_string(),
            "人工确认文案准确".to_string(),
        ]));
    }

    #[test]
    fn final_mark_pass_rejects_writer_self_report_without_bound_reviewer() {
        let fixture = GateFixture::new(vec![PROJECT.to_string()], byte_and_newline_checks());
        fixture.add_worker(
            "writer",
            "work-item:writer",
            vec![PROJECT.to_string()],
            covered_review_evidence(8, false),
        );
        let error = fixture
            .finalize_pass()
            .expect_err("writer self-report cannot satisfy readonly review gate");
        assert!(error.contains("只读复核 worker"), "{error}");
        assert!(error.contains("review_evidence"), "{error}");
    }

    #[test]
    fn final_mark_pass_rejects_unbound_zero_write_near_miss() {
        let fixture = GateFixture::new(vec![PROJECT.to_string()], byte_and_newline_checks());
        fixture.add_worker(
            "unrelated-zero-write",
            "work-item:unrelated-zero-write",
            vec![],
            covered_review_evidence(8, false),
        );
        let error = fixture
            .finalize_pass()
            .expect_err("an unrelated zero-write worker cannot impersonate the reviewer");
        assert!(error.contains("当前授权绑定"), "{error}");
    }

    #[test]
    fn final_mark_pass_rejects_markerless_zero_write_near_miss() {
        let fixture = GateFixture::new(vec![PROJECT.to_string()], byte_and_newline_checks());
        let work_item_id = "work-item:markerless-zero-write";
        fixture.add_worker(
            "markerless-zero-write",
            work_item_id,
            vec![],
            covered_review_evidence(8, false),
        );
        fixture.add_reviewer_task_artifact(
            work_item_id,
            crate::supervisor_session_launcher::supervisor_pilot_readonly_reviewer_task_id(AUTH),
            false,
        );
        let error = fixture
            .finalize_pass()
            .expect_err("markerless zero-write worker cannot impersonate the reviewer");
        assert!(error.contains("当前授权绑定"), "{error}");
    }

    #[test]
    fn final_mark_pass_rejects_incomplete_bound_reviewer_evidence() {
        let fixture = GateFixture::new(vec![PROJECT.to_string()], byte_and_newline_checks());
        let mut report = covered_review_evidence(9, true);
        report["review_evidence"][0]
            .as_object_mut()
            .expect("review evidence object")
            .remove("trailing_newline");
        fixture.add_exact_reviewer("reviewer", "work-item:reviewer", report);
        let error = fixture
            .finalize_pass()
            .expect_err("missing trailing newline evidence must reject pass");
        assert!(error.contains("trailing_newline"), "{error}");
    }

    #[test]
    fn final_mark_allows_covered_bound_reviewer_and_surfaces_conflicts() {
        let fixture = GateFixture::new(vec![PROJECT.to_string()], byte_and_newline_checks());
        fixture.add_exact_reviewer(
            "reviewer",
            "work-item:reviewer",
            covered_review_evidence(9, true),
        );
        let result = fixture
            .finalize_pass()
            .expect("covered reviewer evidence may pass even with a factual mismatch");
        assert_eq!(result["verdict"], "pass");
        assert_eq!(
            result["review_evidence"]["required_fields"],
            json!(["byte_count", "trailing_newline"])
        );
        let advisories = result["review_evidence_advisories"]
            .as_array()
            .expect("advisories array")
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(advisories.contains("byte_count=9"), "{advisories}");
        assert!(advisories.contains("trailing_newline=true"), "{advisories}");
    }

    #[test]
    fn station3b_zero_write_finalize_stays_unchanged_with_byte_family_checks() {
        let fixture = GateFixture::new(vec![], byte_and_newline_checks());
        let result = fixture
            .finalize_pass()
            .expect("zero-write finalization must keep its previous behavior");
        assert_eq!(result["verdict"], "pass");
        assert!(result.get("review_evidence").is_none());
    }
}
