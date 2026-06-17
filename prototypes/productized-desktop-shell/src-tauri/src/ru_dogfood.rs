use crate::{
    AdoptMemoryCandidateInput, CaptureMemoryEventInput, MemoryCaptureCandidateDraft,
    MemoryCaptureSourceRef, MemoryLifecycleStatus, MemoryScope, RecordMemoryCandidateDecisionInput,
};
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuDogfoodConfig {
    pub(crate) workflow_state_path: PathBuf,
    pub(crate) confirmed_workflow_state_path: PathBuf,
    pub(crate) project_root: String,
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    pub(crate) actor_id: String,
    pub(crate) actor_role: String,
    pub(crate) timestamp: String,
    pub(crate) confirmation: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuDogfoodOutput {
    pub(crate) status: String,
    pub(crate) capture_event_id: String,
    pub(crate) observation_id: String,
    pub(crate) candidate_key: String,
    pub(crate) memory_id: String,
    pub(crate) formal_memory_revision: i64,
    pub(crate) candidate_store_revision: i64,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn run_ru_dogfood_memory_adoption(
    config: &RuDogfoodConfig,
) -> Result<RuDogfoodOutput, String> {
    validate_config(config)?;
    let formal_store_before =
        crate::formal_memory_store::load_store(&config.workflow_state_path, &config.timestamp)?;
    let input = capture_input(config);
    let capture = crate::memory_capture_bus::capture_event(
        &config.workflow_state_path,
        &input,
        &config.timestamp,
        &format!("ru2-capture-{}", config.timestamp),
        &format!("ru2-observation-{}", config.timestamp),
        &format!("ru2-candidate-{}", config.timestamp),
    )?;
    let candidate = capture
        .candidate
        .as_ref()
        .ok_or_else(|| "ru_dogfood_missing_candidate_after_capture".to_string())?;
    let observation = capture
        .observation
        .as_ref()
        .ok_or_else(|| "ru_dogfood_missing_observation_after_capture".to_string())?;
    let confirmed_candidate = crate::memory_candidate_store::record_decision(
        &config.workflow_state_path,
        &RecordMemoryCandidateDecisionInput {
            project_root: config.project_root.clone(),
            candidate_key: candidate.candidate_key.clone(),
            requested_status: MemoryLifecycleStatus::CandidateConfirmed,
            actor_id: config.actor_id.clone(),
            actor_role: config.actor_role.clone(),
            reason: "RU2 用户确认该 mariotest dogfood 候选可进入 M2 采纳。".to_string(),
            expected_store_revision: capture.candidate_store_revision,
        },
        &config.timestamp,
        &format!("ru2-candidate-confirmed-{}", config.timestamp),
    )?;
    let adoption = crate::adopt_memory_candidate_to_formal_memory_at(
        &config.workflow_state_path,
        &AdoptMemoryCandidateInput {
            project_root: config.project_root.clone(),
            candidate_key: candidate.candidate_key.clone(),
            actor_id: "project-director:ru-dogfood".to_string(),
            actor_role: "project_director".to_string(),
            adoption_reason:
                "RU2 dogfood 用户在场已确认候选；项目主管按 M2 低风险工作流记忆路径采纳。"
                    .to_string(),
            expected_candidate_store_revision: Some(confirmed_candidate.store_revision),
            expected_formal_store_revision: Some(formal_store_before.revision),
        },
        &config.timestamp,
        &format!("ru2-candidate-adoption-{}", config.timestamp),
        &format!("ru2-formal-adoption-{}", config.timestamp),
    )?;

    Ok(RuDogfoodOutput {
        status: "completed".to_string(),
        capture_event_id: capture.capture_event.capture_event_id,
        observation_id: observation.observation_id.clone(),
        candidate_key: candidate.candidate_key.clone(),
        memory_id: adoption.record.memory_id,
        formal_memory_revision: adoption.formal_store_revision,
        candidate_store_revision: adoption.candidate_store_revision,
        warnings: adoption.warnings,
    })
}

fn validate_config(config: &RuDogfoodConfig) -> Result<(), String> {
    if config.confirmation != "CONFIRMED_USER_PRESENT_2026_06_17" {
        return Err("ru_dogfood_missing_user_present_confirmation".to_string());
    }
    validate_clean_path(&config.workflow_state_path)?;
    validate_clean_path(&config.confirmed_workflow_state_path)?;
    validate_clean_project_root(&config.project_root)?;
    if config.workflow_state_path != config.confirmed_workflow_state_path {
        return Err("ru_dogfood_confirmed_path_mismatch".to_string());
    }
    if !config.workflow_state_path.is_absolute() {
        return Err("ru_dogfood_workflow_state_path_must_be_absolute".to_string());
    }
    if !config.workflow_state_path.exists() {
        return Err("ru_dogfood_workflow_state_path_missing".to_string());
    }
    if config
        .workflow_state_path
        .file_name()
        .and_then(|name| name.to_str())
        != Some("workflow-state.v0.json")
    {
        return Err("ru_dogfood_requires_workflow_state_v0_file".to_string());
    }
    let expected_project_id = crate::project_id(&config.project_root);
    let expected_workflow_id = crate::default_workflow_id(&config.project_root);
    if config.project_id != expected_project_id {
        return Err("ru_dogfood_project_id_mismatch".to_string());
    }
    if config.workflow_id != expected_workflow_id {
        return Err("ru_dogfood_workflow_id_mismatch".to_string());
    }
    validate_registered_project(config)?;
    Ok(())
}

fn validate_registered_project(config: &RuDogfoodConfig) -> Result<(), String> {
    let value = crate::read_workflow_state_value(&config.workflow_state_path)?;
    let warnings = crate::validate_workflow_state(&value);
    if !warnings.is_empty() {
        return Err(format!(
            "ru_dogfood_workflow_state_invalid:{}",
            warnings.join(",")
        ));
    }
    let projects = value
        .get("projects")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "ru_dogfood_workflow_state_missing_projects".to_string())?;
    let project_registered = projects.iter().any(|project| {
        crate::optional_string_from(project, "project_id").as_deref() == Some(&config.project_id)
            && crate::optional_string_from(project, "root_path").as_deref()
                == Some(&config.project_root)
    });
    if !project_registered {
        return Err("ru_dogfood_project_not_registered".to_string());
    }
    let workflows = value
        .get("workflows")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "ru_dogfood_workflow_state_missing_workflows".to_string())?;
    let workflow_registered = workflows.iter().any(|workflow| {
        crate::optional_string_from(workflow, "workflow_id").as_deref() == Some(&config.workflow_id)
            && crate::optional_string_from(workflow, "project_id").as_deref()
                == Some(&config.project_id)
    });
    if !workflow_registered {
        return Err("ru_dogfood_workflow_not_registered".to_string());
    }
    Ok(())
}

fn validate_clean_path(path: &Path) -> Result<(), String> {
    for component in path.components() {
        if let Component::Normal(part) = component {
            let value = part.to_string_lossy().to_ascii_lowercase();
            if is_denied_fragment(&value) {
                return Err(format!("ru_dogfood_denied_path:{}", path.display()));
            }
        }
    }
    Ok(())
}

fn validate_clean_project_root(project_root: &str) -> Result<(), String> {
    if project_root.trim().is_empty() {
        return Err("ru_dogfood_project_root_missing".to_string());
    }
    let path = Path::new(project_root);
    if !path.is_absolute() {
        return Err("ru_dogfood_project_root_must_be_absolute".to_string());
    }
    validate_clean_path(path)
}

fn is_denied_fragment(value: &str) -> bool {
    matches!(
        value,
        ".codex"
            | "secret"
            | "secrets"
            | "token"
            | "tokens"
            | ".env"
            | "keychain"
            | "oauth"
            | "credential"
            | "credentials"
            | "transcript"
            | "transcripts"
            | "prompt"
            | "prompts"
    )
}

fn capture_input(config: &RuDogfoodConfig) -> CaptureMemoryEventInput {
    let scope = MemoryScope {
        scope_id: "scope:ru-dogfood:mario-test-default-workflow".to_string(),
        scope_type: "workflow".to_string(),
        user_id: None,
        project_id: Some(config.project_id.clone()),
        workflow_id: Some(config.workflow_id.clone()),
        session_id: None,
        role_ids: vec!["project_director".to_string(), "user".to_string()],
        document_refs: vec![
            "evidence/2026-06-17-real-use-de-risk-ru1-ru2-blocked-v1.md".to_string(),
            "handoffs/2026-06-17-real-use-de-risk-ru-stage-blocked-result-v1.md".to_string(),
        ],
        permission_policy_ref: Some("RU confirmed-path dogfood; no GUI, no real Codex".to_string()),
        model_export_policy: "local_only".to_string(),
        valid_from: config.timestamp.clone(),
        valid_until: None,
    };
    CaptureMemoryEventInput {
        project_root: config.project_root.clone(),
        project_id: Some(config.project_id.clone()),
        workflow_id: Some(config.workflow_id.clone()),
        workflow_node_id: Some(format!("{}:node:director", config.workflow_id)),
        run_unit_id: Some("run-unit:ru2-dogfood-memory-adoption".to_string()),
        product_command_id: None,
        product_attempt_id: None,
        runtime_log_ref: None,
        audit_refs: vec!["audit:ru2-dogfood-user-present-confirmed".to_string()],
        readback_ref: None,
        task_package_ref: Some(
            "tasks/2026-06-17-real-use-de-risk-ru2-confirmed-path-memory-adoption-and-ru3-conclusion-v1.md"
                .to_string(),
        ),
        memory_packet_ref: None,
        scope,
        source_type: "process_fact_decision".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id: "source:ru2-dogfood-blocker-and-entry-decision".to_string(),
            source_type: "process_fact_decision".to_string(),
            source_id: "ru2-dogfood-confirmed-path-memory-adoption".to_string(),
            project_id: Some(config.project_id.clone()),
            workflow_id: Some(config.workflow_id.clone()),
            workflow_node_id: Some(format!("{}:node:director", config.workflow_id)),
            run_unit_id: Some("run-unit:ru2-dogfood-memory-adoption".to_string()),
            product_command_id: None,
            product_attempt_id: None,
            runtime_log_ref: None,
            audit_ref_id: Some("audit:ru2-dogfood-user-present-confirmed".to_string()),
            readback_ref: None,
            task_package_ref: Some(
                "tasks/2026-06-17-real-use-de-risk-ru2-confirmed-path-memory-adoption-and-ru3-conclusion-v1.md"
                    .to_string(),
            ),
            memory_packet_ref: None,
            evidence_ref: Some(
                "evidence/2026-06-17-real-use-de-risk-ru1-ru2-blocked-v1.md".to_string(),
            ),
            summary:
                "RU dogfood confirmed-path decision for the real mario test workflow.".to_string(),
            sensitive_level: "internal".to_string(),
            created_at: config.timestamp.clone(),
        }],
        summary: "RU dogfood 发现 mario test 已有真实 workflow，但默认 GUI 真机路径会读 Codex state；本次改用 confirmed-path 入口完成记忆闭环。".to_string(),
        evidence_summary: "真实 state root 与 mario test workflow 已只读核实；本次只写工作台记忆 sidecar，不启动 GUI 或真实 Codex。".to_string(),
        sensitivity: "internal".to_string(),
        candidate_policy: "candidate_allowed".to_string(),
        generated_by_role: "project_director".to_string(),
        actor_id: config.actor_id.clone(),
        risk_level: "low".to_string(),
        reason: "RU2 dogfood captures a real project judgment as a reviewable candidate before M2 adoption.".to_string(),
        candidate: Some(MemoryCaptureCandidateDraft {
            memory_type: "workflow_summary".to_string(),
            claim: "mario test RU 发现默认 GUI 真机路径会读 Codex state，记忆闭环需要 confirmed-path 入口。".to_string(),
            body: "在真实 workbench state root 中，mario test 项目和默认 workflow 已存在；RU 去险时发现默认 GUI snapshot 会进入 Codex state 读取路径，和当前安全边界冲突。因此本次真实记忆写入必须通过显式确认的 workflow-state 路径、复用 M2 采纳门完成，不能手写记忆 JSON 或启动默认 GUI 冒充闭环。".to_string(),
            review_reason: "RU2 用户在场确认，把当前真实 dogfood 判断作为第一条正式记忆候选。".to_string(),
            requires_user_confirmation: false,
            actor_role: "project_director".to_string(),
        }),
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        formal_memory_store, memory_candidate_store, memory_capture_bus, observation_store,
    };
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workflow_state_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{label}-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir.join("workflow-state.v0.json")
    }

    fn write_fixture_workflow_state(path: &Path, project_root: &str) {
        let project_id = crate::project_id(project_root);
        let workflow_id = crate::default_workflow_id(project_root);
        let value = json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "workspace_id": "workspace:test",
            "created_at": "2026-06-17T00:00:00Z",
            "updated_at": "2026-06-17T00:00:00Z",
            "source_kind": "workspace_state",
            "permission_level": "user_confirmed_write",
            "projects": [{
                "project_id": project_id,
                "root_path": project_root,
                "display_name": "mario test",
                "source_kind": "workspace_state",
                "permission_level": "user_confirmed_write",
                "created_at": "2026-06-17T00:00:00Z",
                "updated_at": "2026-06-17T00:00:00Z",
                "warnings": []
            }],
            "agent_adapters": [],
            "workflows": [{
                "workflow_id": workflow_id,
                "project_id": project_id,
                "title": "mario test RU fixture workflow",
                "state": "draft",
                "source_kind": "workspace_state",
                "permission_level": "user_confirmed_write",
                "workflow_version": 1,
                "entry_node_id": format!("{workflow_id}:node:director"),
                "model_policy": "codex_threads_user_confirmed",
                "created_at": "2026-06-17T00:00:00Z",
                "updated_at": "2026-06-17T00:00:00Z",
                "warnings": ["real_codex_resume_requires_separate_user_approval"]
            }],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "workflow_node_session_bindings": [],
            "workflow_node_dispatches": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": []
        });
        fs::write(path, serde_json::to_string_pretty(&value).expect("json"))
            .expect("write fixture");
    }

    fn fixture_config(path: PathBuf, project_root: &str) -> RuDogfoodConfig {
        RuDogfoodConfig {
            workflow_state_path: path.clone(),
            confirmed_workflow_state_path: path,
            project_root: project_root.to_string(),
            project_id: crate::project_id(project_root),
            workflow_id: crate::default_workflow_id(project_root),
            actor_id: "user:ru-dogfood-test".to_string(),
            actor_role: "user".to_string(),
            timestamp: "2026-06-17T09:00:00Z".to_string(),
            confirmation: "CONFIRMED_USER_PRESENT_2026_06_17".to_string(),
        }
    }

    #[test]
    fn ru_dogfood_rejects_unconfirmed_workflow_state_path() {
        let workflow_state_path = temp_workflow_state_path("ru-dogfood-mismatch");
        write_fixture_workflow_state(&workflow_state_path, "/tmp/ru-dogfood-mismatch");
        let mut config = fixture_config(workflow_state_path, "/tmp/ru-dogfood-mismatch");
        config.confirmed_workflow_state_path =
            temp_workflow_state_path("ru-dogfood-other-confirmed");

        let err = run_ru_dogfood_memory_adoption(&config).expect_err("mismatched path rejected");

        assert!(err.contains("ru_dogfood_confirmed_path_mismatch"));
        assert!(
            !memory_capture_bus::sidecar_path(&config.workflow_state_path)
                .expect("capture sidecar path")
                .exists()
        );
    }

    #[test]
    fn ru_dogfood_rejects_denied_codex_path() {
        let mut config = fixture_config(
            PathBuf::from("/tmp/ru-dogfood/.codex/workflow-state.v0.json"),
            "/tmp/ru-dogfood-project",
        );
        config.confirmed_workflow_state_path = config.workflow_state_path.clone();

        let err = run_ru_dogfood_memory_adoption(&config).expect_err("denied path rejected");

        assert!(err.contains("ru_dogfood_denied_path"));
    }

    #[test]
    fn ru_dogfood_confirmed_fixture_writes_via_m2_adoption() {
        let workflow_state_path = temp_workflow_state_path("ru-dogfood-success");
        let project_root = "/tmp/ru-dogfood-success";
        write_fixture_workflow_state(&workflow_state_path, project_root);
        let config = fixture_config(workflow_state_path.clone(), project_root);

        let output = run_ru_dogfood_memory_adoption(&config).expect("ru dogfood adoption");

        assert_eq!(output.status, "completed");
        assert!(output
            .warnings
            .iter()
            .any(|warning| warning == "memory_candidate_adopted_to_formal_memory"));
        let capture_store = memory_capture_bus::load_store(&workflow_state_path, &config.timestamp)
            .expect("capture store");
        let observation_store =
            observation_store::load_store(&workflow_state_path, &config.timestamp)
                .expect("observation store");
        let candidate_store =
            memory_candidate_store::load_store(&workflow_state_path, &config.timestamp)
                .expect("candidate store");
        let formal_store = formal_memory_store::load_store(&workflow_state_path, &config.timestamp)
            .expect("formal store");

        assert_eq!(capture_store.events.len(), 1);
        assert_eq!(observation_store.observations.len(), 1);
        assert_eq!(candidate_store.candidates.len(), 1);
        assert_eq!(formal_store.records.len(), 1);
        assert_eq!(
            candidate_store.candidates[0]
                .adoption
                .as_ref()
                .expect("adoption link")
                .adopted_memory_id,
            formal_store.records[0].memory_id
        );
        assert_eq!(formal_store.records[0].claim, "mario test RU 发现默认 GUI 真机路径会读 Codex state，记忆闭环需要 confirmed-path 入口。");
    }

    #[test]
    #[ignore = "RU2 real dogfood runner requires explicit user-present env confirmation"]
    fn r3_ru2_dogfood_confirmed_paths_requires_env_authorization() {
        let confirmation = std::env::var("R3_RU2_DOGFOOD_CONFIRM").expect("R3_RU2_DOGFOOD_CONFIRM");
        assert_eq!(confirmation, "CONFIRMED_USER_PRESENT_2026_06_17");
        let workflow_state_path =
            PathBuf::from(std::env::var("R3_RU2_WORKFLOW_STATE_PATH").expect("state path"));
        let confirmed_workflow_state_path = PathBuf::from(
            std::env::var("R3_RU2_CONFIRMED_WORKFLOW_STATE_PATH").expect("confirmed state path"),
        );
        let project_root = std::env::var("R3_RU2_PROJECT_ROOT").expect("project root");
        let config = RuDogfoodConfig {
            workflow_state_path,
            confirmed_workflow_state_path,
            project_id: std::env::var("R3_RU2_PROJECT_ID").expect("project id"),
            workflow_id: std::env::var("R3_RU2_WORKFLOW_ID").expect("workflow id"),
            project_root,
            actor_id: "user:ru-dogfood".to_string(),
            actor_role: "user".to_string(),
            timestamp: crate::unix_timestamp_string(),
            confirmation,
        };

        let output = run_ru_dogfood_memory_adoption(&config).expect("real ru2 dogfood adoption");

        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": output.status,
                "capture_event_id": output.capture_event_id,
                "observation_id": output.observation_id,
                "candidate_key": output.candidate_key,
                "memory_id": output.memory_id,
                "formal_memory_revision": output.formal_memory_revision,
                "candidate_store_revision": output.candidate_store_revision,
                "warnings": output.warnings
            }))
            .expect("json")
        );
        assert_eq!(output.status, "completed");
        assert!(output.memory_id.starts_with("mem:v1:"));
    }
}
